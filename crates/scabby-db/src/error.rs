//! Storage errors and their translation into domain errors.

use scabby_core::Error;

/// A failure from the storage layer.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// Postgres refused the statement or the connection.
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// A migration could not be applied.
    #[error("migration failed: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    /// The row is not there.
    #[error("{resource} not found: {key}")]
    NotFound {
        /// Kind of thing that was looked up.
        resource: &'static str,
        /// What was looked up.
        key: String,
    },

    /// The value is acceptable to Postgres and not to the domain.
    #[error("{0}")]
    Domain(#[from] Error),
}

impl DbError {
    /// Names a row that should have existed.
    pub fn not_found(resource: &'static str, key: impl std::fmt::Display) -> Self {
        Self::NotFound {
            resource,
            key: key.to_string(),
        }
    }

    /// Turns a unique-violation into a domain conflict, so callers can report which name collided.
    ///
    /// Any other failure is passed through untouched: a broken connection must not be reported as a
    /// duplicate name, and a foreign-key violation is a validation failure rather than a conflict.
    pub fn or_conflict(self, resource: &'static str, key: &str) -> Self {
        match &self {
            Self::Sqlx(sqlx::Error::Database(database))
                if database.code().as_deref() == Some("23505") =>
            {
                Self::Domain(Error::conflict(resource, key))
            }
            Self::Sqlx(sqlx::Error::Database(database))
                if database.code().as_deref() == Some("23503") =>
            {
                Self::Domain(Error::validation(
                    "reference",
                    format!("{resource} points at a row that no longer exists"),
                ))
            }
            _ => self,
        }
    }
}

impl From<DbError> for Error {
    fn from(value: DbError) -> Self {
        match value {
            DbError::Sqlx(error) => Error::internal(format!("database: {error}")),
            DbError::Migrate(error) => Error::internal(format!("migration: {error}")),
            DbError::NotFound { resource, key } => Error::not_found(resource, key),
            DbError::Domain(error) => error,
        }
    }
}
