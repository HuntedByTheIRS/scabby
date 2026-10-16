//! The command line: settings, and the four things the binary can be asked to do.

use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use scabby_core::config::Registration;
use url::Url;

/// Log filter used when nothing overrides it.
pub const DEFAULT_LOG: &str = "info";

/// One binary that serves the API, the user interface, and git.
#[derive(Debug, Parser)]
#[command(
    name = "scabby",
    version,
    about = "Self-hosted Git hosting: repositories, issues, and a REST API.",
    long_about = None,
)]
pub struct Cli {
    /// Instance settings, which may also come from the environment.
    #[command(flatten)]
    pub settings: Settings,
    /// What to run. Serving is the default when nothing is given.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Everything that can be configured.
///
/// Each of these is a flag, a `SCABBY_*` environment variable, and a key in `scabby.toml`, and each
/// is `global` so it can be written before or after the subcommand: `scabby serve --frontend-dir
/// web/build` reads the way someone would type it.
#[derive(Debug, Args)]
pub struct Settings {
    /// Path to a TOML configuration file. Without it, `./scabby.toml` is used when it exists.
    #[arg(short, long, global = true, env = "SCABBY_CONFIG", value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Address to listen on.
    #[arg(long, global = true, env = "SCABBY_LISTEN", value_name = "ADDR")]
    pub listen: Option<SocketAddr>,

    /// PostgreSQL connection string.
    #[arg(long, global = true, env = "SCABBY_DATABASE_URL", value_name = "URL")]
    pub database_url: Option<String>,

    /// Root directory for bare repositories.
    #[arg(long, global = true, env = "SCABBY_DATA_DIR", value_name = "PATH")]
    pub data_dir: Option<PathBuf>,

    /// Public origin, used for clone URLs, cookies, and links.
    #[arg(long, global = true, env = "SCABBY_BASE_URL", value_name = "URL")]
    pub base_url: Option<Url>,

    /// Serve the user interface from this directory instead of the embedded copy.
    #[arg(long, global = true, env = "SCABBY_FRONTEND_DIR", value_name = "PATH")]
    pub frontend_dir: Option<PathBuf>,

    /// Who may create an account.
    #[arg(long, global = true, env = "SCABBY_REGISTRATION", value_name = "MODE")]
    pub registration: Option<RegistrationMode>,

    /// git binary used for the transport and for `git init`.
    #[arg(
        long,
        global = true,
        env = "SCABBY_GIT",
        value_name = "PATH",
        default_value = "git"
    )]
    pub git: PathBuf,

    /// Tracing filter, such as `info` or `scabby_web=debug`.
    #[arg(
        long,
        global = true,
        env = "SCABBY_LOG",
        value_name = "FILTER",
        default_value = DEFAULT_LOG
    )]
    pub log: String,

    /// Ceiling on database connections.
    #[arg(long, global = true, env = "SCABBY_MAX_CONNECTIONS", value_name = "N")]
    pub max_connections: Option<u32>,

    /// Apply pending migrations when the server starts.
    #[arg(
        long,
        global = true,
        env = "SCABBY_MIGRATE_ON_START",
        value_name = "BOOL"
    )]
    pub migrate_on_start: Option<bool>,
}

impl Settings {
    /// The log filter.
    ///
    /// `RUST_LOG` is honoured in addition to `SCABBY_LOG` and `--log`, which is what every other
    /// Rust service does and therefore what an operator will try. clap cannot tell a flag from an
    /// environment value for the same argument, so the rule is: anything that is not the default
    /// came from someone, and the default falls back to `RUST_LOG`.
    pub fn log_filter(&self) -> String {
        if self.log != DEFAULT_LOG {
            return self.log.clone();
        }

        match std::env::var("RUST_LOG") {
            Ok(from_env) if !from_env.trim().is_empty() => from_env,
            _ => self.log.clone(),
        }
    }
}

/// Who may create an account, as a flag value.
///
/// The domain type lives in `scabby-core`, which must not depend on a command line parser, so the
/// two are bridged here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RegistrationMode {
    /// Anyone may sign up.
    Open,
    /// Accounts are created by an administrator.
    Closed,
}

impl From<RegistrationMode> for Registration {
    fn from(mode: RegistrationMode) -> Self {
        match mode {
            RegistrationMode::Open => Self::Open,
            RegistrationMode::Closed => Self::Closed,
        }
    }
}

/// What to run.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the HTTP server. This is the default.
    Serve,
    /// Apply database migrations and exit.
    Migrate,
    /// Administrative commands.
    #[command(subcommand)]
    Admin(AdminCommand),
    /// Called by the git hooks installed in each repository, not by people.
    Hook(HookCommand),
}

/// Commands that act on the instance rather than through it.
#[derive(Debug, Subcommand)]
pub enum AdminCommand {
    /// Create an account.
    CreateUser {
        /// Handle, which is also the first half of every repository path.
        #[arg(long, value_name = "NAME")]
        username: String,
        /// Email address.
        #[arg(long, value_name = "ADDRESS")]
        email: String,
        /// Grant administrator rights.
        #[arg(long)]
        admin: bool,
        /// Read the password from stdin instead of generating one.
        #[arg(long)]
        password_stdin: bool,
    },
    /// Rewrite the hooks in every repository, which is what an upgrade needs after the binary moves.
    InstallHooks,
}

/// Hooks git runs, all of which call back into this binary.
#[derive(Debug, Args)]
pub struct HookCommand {
    /// Which hook is running.
    #[command(subcommand)]
    pub kind: HookKind,
}

/// The hooks Scabby installs.
#[derive(Debug, Subcommand)]
pub enum HookKind {
    /// Runs after a push has been accepted.
    PostReceive {
        /// Repository the push landed in, as `owner/name`.
        #[arg(long, value_name = "OWNER/NAME")]
        repo: String,
    },
    /// Runs before a push is accepted.
    PreReceive {
        /// Repository the push is aimed at, as `owner/name`.
        #[arg(long, value_name = "OWNER/NAME")]
        repo: String,
    },
}
