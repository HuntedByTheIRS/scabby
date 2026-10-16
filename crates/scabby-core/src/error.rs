//! The error type every crate funnels into.

use std::fmt;

/// Result alias used across the workspace.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// A domain error, mapped to an HTTP status by the web layer.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The resource does not exist, or the caller may not know that it does.
    #[error("{resource} not found: {key}")]
    NotFound {
        /// Kind of thing that was looked up, such as `repository`.
        resource: &'static str,
        /// What was looked up, such as `ada/scabby`.
        key: String,
    },

    /// The resource already exists.
    #[error("{resource} already exists: {key}")]
    Conflict {
        /// Kind of thing that collided.
        resource: &'static str,
        /// The name that collided.
        key: String,
    },

    /// The request was well formed and the value is not acceptable.
    #[error("{field}: {message}")]
    Validation {
        /// Field that failed, reported as `errors[].field`.
        field: &'static str,
        /// Explanation shown to the caller.
        message: String,
    },

    /// No credentials, or credentials that do not check out.
    #[error("authentication required")]
    Unauthorized,

    /// Authenticated, but not allowed to do this.
    #[error("{0}")]
    Forbidden(String),

    /// Something failed that the caller cannot fix.
    #[error("{0}")]
    Internal(String),
}

impl Error {
    /// Build a not-found error for a named resource.
    pub fn not_found(resource: &'static str, key: impl fmt::Display) -> Self {
        Self::NotFound {
            resource,
            key: key.to_string(),
        }
    }

    /// Build a conflict for a resource that already exists.
    pub fn conflict(resource: &'static str, key: impl fmt::Display) -> Self {
        Self::Conflict {
            resource,
            key: key.to_string(),
        }
    }

    /// Build a validation failure for one field.
    pub fn validation(field: &'static str, message: impl Into<String>) -> Self {
        Self::Validation {
            field,
            message: message.into(),
        }
    }

    /// Build a permission failure.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden(message.into())
    }

    /// Wrap a failure that is not the caller's fault.
    pub fn internal(error: impl fmt::Display) -> Self {
        Self::Internal(error.to_string())
    }

    /// Machine-readable code for `errors[].code` in an API error body.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "not_found",
            Self::Conflict { .. } => "already_exists",
            Self::Validation { .. } => "invalid",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::Internal(_) => "internal",
        }
    }

    /// Resource name for `errors[].resource`, when the error names one.
    pub fn resource(&self) -> Option<&'static str> {
        match self {
            Self::NotFound { resource, .. } | Self::Conflict { resource, .. } => Some(resource),
            _ => None,
        }
    }

    /// Field name for `errors[].field`, when the error names one.
    pub fn field(&self) -> Option<&'static str> {
        match self {
            Self::Validation { field, .. } => Some(field),
            _ => None,
        }
    }

    /// Whether this is a lookup miss, which the web layer reports as `404`.
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }
}
