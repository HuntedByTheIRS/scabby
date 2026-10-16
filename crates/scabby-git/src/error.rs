//! Git failures and their translation into domain errors.

use scabby_core::Error;

/// Result alias for git operations.
pub type Result<T, E = GitError> = std::result::Result<T, E>;

/// A failure from the git layer.
///
/// gix reports its failures through a different error type per call, so those are carried as text
/// and the variants that matter to the API get a name of their own.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// The filesystem refused.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A gix call failed.
    #[error("git error: {0}")]
    Git(String),

    /// The database knows about a repository the disk does not.
    #[error("repository {slug} has no directory on disk")]
    MissingRepository {
        /// `owner/name` that was looked for.
        slug: String,
    },

    /// A revision string did not resolve to an object.
    #[error("unknown revision {revision}")]
    UnknownRevision {
        /// What the caller asked for.
        revision: String,
    },

    /// A path does not exist in that revision's tree.
    #[error("{path} does not exist at {revision}")]
    MissingPath {
        /// Path within the repository.
        path: String,
        /// Revision it was looked up at.
        revision: String,
    },

    /// A git command failed.
    #[error("git {command} exited with {status}: {message}")]
    Command {
        /// Subcommand that ran.
        command: String,
        /// Exit code.
        status: i32,
        /// Whatever it wrote to stderr, or a note that nothing was captured.
        message: String,
    },

    /// The domain layer rejected the input.
    #[error(transparent)]
    Domain(#[from] Error),
}

impl GitError {
    /// Wrap a gix failure with its own message.
    pub fn from_git(error: impl std::fmt::Display) -> Self {
        Self::Git(error.to_string())
    }
}

impl From<GitError> for Error {
    fn from(value: GitError) -> Self {
        match value {
            GitError::MissingRepository { slug } => Error::not_found("repository", slug),
            GitError::UnknownRevision { revision } => {
                Error::validation("revision", format!("unknown revision {revision}"))
            }
            GitError::MissingPath { path, revision } => {
                Error::not_found("path", format!("{revision}:{path}"))
            }
            GitError::Domain(error) => error,
            other => Error::internal(other.to_string()),
        }
    }
}
