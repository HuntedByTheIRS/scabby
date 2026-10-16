//! Version and health.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Serialize;

use crate::AppState;

/// Routes that describe the instance rather than a resource.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/version", get(version))
        .route("/health", get(health))
}

/// What the API says about itself.
#[derive(Debug, Serialize)]
pub struct Version {
    /// Server name.
    pub name: &'static str,
    /// Build version.
    pub version: &'static str,
    /// API version, which is the prefix every route below lives under.
    pub api_version: &'static str,
}

/// `GET /api/v1/version`
pub async fn version() -> Json<Version> {
    Json(Version {
        name: "scabby",
        version: env!("CARGO_PKG_VERSION"),
        api_version: "v1",
    })
}

/// The health body.
#[derive(Debug, Serialize)]
pub struct Health {
    /// `ok` when everything answered, `degraded` when Postgres did not.
    pub status: &'static str,
    /// `ok` or `unreachable`.
    pub database: &'static str,
    /// Seconds since the process started.
    pub uptime_seconds: u64,
}

/// `GET /api/v1/health`
///
/// A failing database is reported as `503` with a body rather than as a bare error, because the
/// caller is usually a load balancer deciding whether to send traffic here.
pub async fn health(State(state): State<AppState>) -> Response {
    let database_ok = state.db.is_healthy().await;
    let body = Health {
        status: if database_ok { "ok" } else { "degraded" },
        database: if database_ok { "ok" } else { "unreachable" },
        uptime_seconds: state.started_at.elapsed().as_secs(),
    };

    let status = if database_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(body)).into_response()
}

/// `GET /healthz`, unversioned and deliberately trivial, for a load balancer that wants a word.
pub async fn healthz() -> &'static str {
    "ok"
}
