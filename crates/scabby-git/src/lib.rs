//! Bare repository storage and the git smart-HTTP endpoints.
//!
//! Object reads go through `gix`, the pure Rust git implementation: commits, trees, blobs, and refs
//! are read without starting a process. Everything that is genuinely a protocol or a patch is
//! handled by the `git` binary instead, because reimplementing pack negotiation or a diff engine is
//! how a git host ends up subtly wrong. The split is deliberate:
//!
//! - `read` and `store`: `gix`, in process.
//! - `transport`: `git upload-pack` and `git receive-pack`, one process per request.
//! - `diff`: `git diff-tree`, so the line counts match what a developer sees in a terminal.
//! - `store::stats_all`: rayon, because the dashboard wants every repository measured at once.

pub mod diff;
pub mod error;
pub mod hooks;
pub mod read;
pub mod store;
pub mod transport;

pub use error::{GitError, Result};
pub use store::GitStore;
pub use transport::{Service, SmartHttp};
