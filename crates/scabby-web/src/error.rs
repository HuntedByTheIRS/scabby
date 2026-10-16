//! Errors as HTTP responses.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use scabby_core::Error;
use scabby_db::DbError;
use scabby_git::GitError;
use serde::Serialize;

/// One entry of the `errors` array in an API error body.
#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorEntry {
    /// What kind of thing the error is about, when that is known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<&'static str>,
    /// Which field failed, for a validation error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<&'static str>,
    /// Machine-readable code.
    pub code: &'static str,
}

/// The error body every failing API request returns.
#[derive(Debug, Clone, Serialize)]
pub struct ApiErrorBody {
    /// Human-readable summary, in the shape GitHub uses.
    pub message: String,
    /// HTTP status, repeated in the body so a logged response is self-describing.
    pub status: u16,
    /// Detail, present only when there is something actionable to say.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<ApiErrorEntry>,
}

/// Anything a handler can fail with.
///
/// The storage and git layers convert into the domain error on the way in, so the status mapping
/// lives in one place: `scabby_core::Error` already knows which failure is a 404 and which is a 500.
#[derive(Debug, thiserror::Error)]
pub enum WebError {
    /// A domain rule was broken, including anything storage or git reported.
    #[error(transparent)]
    Domain(#[from] Error),

    /// No credentials, or credentials that do not check out.
    #[error("authentication required")]
    Unauthorized {
        /// Whether to answer with a `WWW-Authenticate` challenge, which git clients need to prompt.
        challenge: bool,
    },

    /// The request body was not acceptable.
    #[error("{0}")]
    BadRequest(String),
}

impl From<DbError> for WebError {
    fn from(error: DbError) -> Self {
        Self::Domain(Error::from(error))
    }
}

impl From<GitError> for WebError {
    fn from(error: GitError) -> Self {
        Self::Domain(Error::from(error))
    }
}

impl WebError {
    /// Status this error maps to.
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Domain(error) => match error {
                Error::NotFound { .. } => StatusCode::NOT_FOUND,
                Error::Conflict { .. } => StatusCode::CONFLICT,
                Error::Validation { .. } => StatusCode::UNPROCESSABLE_ENTITY,
                Error::Unauthorized => StatusCode::UNAUTHORIZED,
                Error::Forbidden(_) => StatusCode::FORBIDDEN,
                Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            },
            Self::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }

    /// A permission failure, phrased for the resource at hand.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Domain(Error::forbidden(message))
    }

    /// The response an unauthenticated caller gets. `challenge` decides whether a git client is told
    /// to ask for a password.
    pub fn unauthorized(challenge: bool) -> Self {
        Self::Unauthorized { challenge }
    }

    /// A path or resource that does not exist.
    pub fn not_found(resource: &'static str, key: impl Into<String>) -> Self {
        Self::Domain(Error::not_found(resource, key.into()))
    }

    /// A validation failure for one field.
    pub fn validation(field: &'static str, message: impl Into<String>) -> Self {
        Self::Domain(Error::validation(field, message))
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = match &self {
            Self::Domain(error) => error.to_string(),
            Self::Unauthorized { .. } => "Requires authentication".to_owned(),
            Self::BadRequest(message) => message.clone(),
        };

        // A 500 is a bug on this side: the operator gets the reason in the log, the client gets a
        // plain message with no internals in it.
        if status == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(error = %self, "request failed");
        }

        let entries = match &self {
            Self::Domain(error) => vec![ApiErrorEntry {
                resource: error.resource(),
                field: error.field(),
                code: error.code(),
            }],
            _ => Vec::new(),
        };

        let body = ApiErrorBody {
            message,
            status: status.as_u16(),
            errors: entries,
        };
        let mut response = (status, Json(body)).into_response();

        if matches!(self, Self::Unauthorized { challenge: true }) {
            response.headers_mut().insert(
                axum::http::header::WWW_AUTHENTICATE,
                axum::http::HeaderValue::from_static("Basic realm=\"Scabby\""),
            );
        }
        response
    }
}
