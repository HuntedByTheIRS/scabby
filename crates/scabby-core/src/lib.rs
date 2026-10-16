//! Domain types, identifiers, errors, and configuration shared by every Scabby crate.
//!
//! Nothing here talks to Postgres, the filesystem, or HTTP. The database and git layers convert
//! their own failures into [`Error`] and hand back the types defined here, which keeps the API
//! layer free of storage details.

pub mod auth;
pub mod config;
pub mod error;
pub mod event;
pub mod git;
pub mod ids;
pub mod issue;
pub mod names;
pub mod pagination;
pub mod repo;
pub mod secret;
pub mod timestamp;
pub mod user;

pub use error::{Error, Result};
pub use names::{Name, Slug};
