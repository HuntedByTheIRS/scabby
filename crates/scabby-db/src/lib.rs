//! Postgres storage for Scabby.
//!
//! One module per resource, each holding SQL in one place and converting rows into the types from
//! `scabby-core`. Queries are checked at runtime rather than with the `query!` macros, so building
//! the workspace never needs a live database; the integration tests in `tests/postgres.rs` are what
//! exercise them, against `SCABBY_TEST_DATABASE_URL`.

pub mod error;
pub mod events;
pub mod issues;
pub mod repos;
mod rows;
mod sql;
pub mod tokens;
pub mod users;

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub use error::DbError;

/// Result alias for storage operations.
pub type Result<T, E = DbError> = std::result::Result<T, E>;

/// A connection pool to the application database.
#[derive(Clone, Debug)]
pub struct Db {
    pool: PgPool,
}

impl Db {
    /// Open a pool, verifying that the database answers before returning.
    pub async fn connect(url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    /// The underlying pool, for the migration runner and the health check.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Apply every migration that has not run yet. Migrations are embedded in the binary, so a
    /// deployment never has to ship the SQL alongside it.
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Postgres version string, used by the health endpoint to prove the connection is real.
    pub async fn server_version(&self) -> Result<String> {
        let version: String = sqlx::query_scalar("select version()")
            .fetch_one(&self.pool)
            .await?;
        Ok(version)
    }

    /// Whether the database answers a trivial query.
    pub async fn is_healthy(&self) -> bool {
        sqlx::query_scalar::<_, i32>("select 1")
            .fetch_one(&self.pool)
            .await
            .is_ok()
    }

    /// Close the pool, waiting for in-flight queries.
    pub async fn close(&self) {
        self.pool.close().await;
    }
}
