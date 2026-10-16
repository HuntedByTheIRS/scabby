//! Git smart-HTTP, served by the git binary.
//!
//! Scabby does not reimplement the wire protocol. A request runs `git upload-pack --stateless-rpc`
//! or `git receive-pack --stateless-rpc` with the bare repository as its working directory, the
//! request body on stdin, and the response on stdout, which is the same division of labour
//! `git http-backend` uses. The difference is who decides: the access check happens in Rust before
//! a single byte is handed to git, so an unauthorized push never reaches the pack parser.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use tokio::io::AsyncReadExt;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::Result;
use crate::error::GitError;

/// Which half of the smart protocol a request is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Service {
    /// Fetch: `git-upload-pack`.
    UploadPack,
    /// Push: `git-receive-pack`.
    ReceivePack,
}

impl Service {
    /// The name git uses on the wire and in content types.
    pub const fn name(self) -> &'static str {
        match self {
            Self::UploadPack => "git-upload-pack",
            Self::ReceivePack => "git-receive-pack",
        }
    }

    /// The same service as a subcommand of `git`.
    ///
    /// `git-upload-pack` is a program of its own, not a subcommand, so it has to be spelled
    /// `git upload-pack` when the arguments go to the `git` launcher.
    pub const fn subcommand(self) -> &'static str {
        match self {
            Self::UploadPack => "upload-pack",
            Self::ReceivePack => "receive-pack",
        }
    }

    /// Parse the `service=` query parameter of an advertise request.
    pub fn parse(service: &str) -> Option<Self> {
        match service {
            "git-upload-pack" => Some(Self::UploadPack),
            "git-receive-pack" => Some(Self::ReceivePack),
            _ => None,
        }
    }

    /// Content type of the `info/refs` response.
    pub fn advertisement_content_type(self) -> String {
        format!("application/x-{}-advertisement", self.name())
    }

    /// Content type of the RPC response.
    pub fn result_content_type(self) -> String {
        format!("application/x-{}-result", self.name())
    }
}

/// The pkt-line that opens an advertisement response, as the protocol requires.
pub fn advertisement_header(service: Service) -> String {
    let line = format!("# service={}\n", service.name());
    // The length prefix counts the four bytes of the prefix itself.
    format!("{:04x}{line}0000", line.len() + 4)
}

/// A running git service process, wired to the pipes of one HTTP exchange.
///
/// The process is not awaited here: the caller pumps the request body into [`Session::take_stdin`],
/// streams [`Session::take_stdout`] back to the client, and then awaits [`Session::finish`] so a git
/// failure ends up in the log instead of disappearing.
pub struct Session {
    child: Child,
    command: String,
}

impl Session {
    /// Take the process's stdin, so the caller can write the request body into it.
    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin.take()
    }

    /// Take the process's stdout, which is the response body.
    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }

    /// Wait for git to exit, reporting a non-zero status with whatever it wrote to stderr.
    pub async fn finish(mut self) -> Result<()> {
        let mut stderr = String::new();
        if let Some(mut pipe) = self.child.stderr.take() {
            let _ = pipe.read_to_string(&mut stderr).await;
        }

        let status = self.child.wait().await?;
        if status.success() {
            return Ok(());
        }

        let message = if stderr.trim().is_empty() {
            "nothing was captured on stderr".to_owned()
        } else {
            stderr.trim().to_owned()
        };

        Err(GitError::Command {
            command: self.command,
            status: status.code().unwrap_or(-1),
            message,
        })
    }

    /// Stop the process, for a request the client abandoned.
    pub async fn kill(mut self) {
        let _ = self.child.kill().await;
    }
}

/// Starts git services against a repository directory.
#[derive(Debug, Clone)]
pub struct SmartHttp {
    git: PathBuf,
    env: Vec<(String, String)>,
}

impl Default for SmartHttp {
    fn default() -> Self {
        Self::new()
    }
}

impl SmartHttp {
    /// Use `git` from `PATH`.
    pub fn new() -> Self {
        Self {
            git: PathBuf::from("git"),
            env: Vec::new(),
        }
    }

    /// Use a specific git binary.
    #[must_use]
    pub fn with_git(mut self, git: impl Into<PathBuf>) -> Self {
        self.git = git.into();
        self
    }

    /// Pass an environment variable to every git process this transport starts.
    ///
    /// This exists for the hooks. `git receive-pack` runs `post-receive`, which calls back into the
    /// `scabby` binary to record the push, and that callback needs the database and data directory
    /// the server was configured with. A child inherits the parent's environment but not its
    /// resolved configuration file, so the values are set here, at the moment the child is created.
    #[must_use]
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Produce the `info/refs` body for a fetch or a push.
    ///
    /// `protocol` carries the `Git-Protocol` request header when the client asked for v2, which git
    /// needs to advertise the right capability set.
    pub async fn advertise(
        &self,
        repo_dir: &Path,
        service: Service,
        protocol: Option<&str>,
    ) -> Result<Vec<u8>> {
        // The advertisement is the same service in stateless mode with the ref list turned on.
        // `--advertise-refs` is the long-standing spelling for this request, so it works on git
        // versions that predate `--http-backend-info-refs`.
        let mut command = self.base_command(repo_dir, protocol);
        command
            .arg(service.subcommand())
            .arg("--stateless-rpc")
            .arg("--advertise-refs")
            .arg(".");

        let output = command.output().await?;
        if !output.status.success() {
            return Err(GitError::Command {
                command: format!(
                    "git {} --stateless-rpc --advertise-refs",
                    service.subcommand()
                ),
                status: output.status.code().unwrap_or(-1),
                message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }

        let mut body = advertisement_header(service).into_bytes();
        body.extend_from_slice(&output.stdout);
        Ok(body)
    }

    /// Start the stateless RPC for a request body.
    pub fn start(
        &self,
        repo_dir: &Path,
        service: Service,
        protocol: Option<&str>,
        actor: Option<&str>,
    ) -> Result<Session> {
        let mut command = self.base_command(repo_dir, protocol);
        // The working directory is the repository, so the directory argument is the current one.
        command
            .arg(service.subcommand())
            .arg("--stateless-rpc")
            .arg(".");

        // git passes this into hooks, so a push can be attributed without the hook guessing.
        if let Some(actor) = actor {
            command.env("REMOTE_USER", actor);
        }

        // Keep the process's stdin and stdout in the caller's hands; stderr is ours to report.
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let child = command.spawn().map_err(GitError::from_git)?;
        Ok(Session {
            child,
            command: format!("git {} --stateless-rpc", service.subcommand()),
        })
    }

    fn base_command(&self, repo_dir: &Path, protocol: Option<&str>) -> Command {
        let mut command = Command::new(&self.git);
        command.current_dir(repo_dir);
        if let Some(protocol) = protocol {
            command.env("GIT_PROTOCOL", protocol);
        }
        for (key, value) in &self.env {
            command.env(key, value);
        }
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advertisement_preamble_is_a_valid_pkt_line() {
        let header = advertisement_header(Service::UploadPack);
        assert_eq!(&header[..4], "001e");
        assert!(header.ends_with("0000"));
        assert!(header.contains("# service=git-upload-pack\n"));
    }

    #[test]
    fn services_parse_and_name_themselves() {
        assert_eq!(Service::parse("git-upload-pack"), Some(Service::UploadPack));
        assert_eq!(
            Service::parse("git-receive-pack"),
            Some(Service::ReceivePack)
        );
        assert_eq!(Service::parse("git-nonsense"), None);
        // The wire name and the git subcommand differ: `git-upload-pack` is a program, and the
        // launcher wants `git upload-pack`.
        assert_eq!(Service::UploadPack.subcommand(), "upload-pack");
        assert_eq!(Service::ReceivePack.subcommand(), "receive-pack");
        assert_eq!(
            Service::UploadPack.result_content_type(),
            "application/x-git-upload-pack-result"
        );
    }
}
