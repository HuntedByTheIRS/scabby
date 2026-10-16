//! The HTTP surface: the REST API, git smart-HTTP, session auth, and the embedded UI.
//!
//! Every screen in the browser talks to `/api/v1`, so anything the UI can do is scriptable. The only
//! routes outside the API are the git endpoints a git client expects to find at `owner/name.git`,
//! and the static files.

pub mod api;
mod assets;
mod auth;
mod error;
mod git_http;
mod middleware;

use std::sync::Arc;
use std::time::Instant;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::get;
use scabby_core::config::Config;
use scabby_db::Db;
use scabby_git::{GitStore, SmartHttp};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::compression::CompressionLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

pub use error::WebError;

/// Everything a request handler needs, cloned per request.
#[derive(Clone)]
pub struct AppState {
    /// Instance configuration.
    pub config: Arc<Config>,
    /// Postgres pool and migrations.
    pub db: Db,
    /// Bare repositories on disk.
    pub git: GitStore,
    /// Starts `git upload-pack` / `git receive-pack` for a request.
    pub transport: SmartHttp,
    /// When the process started, for the health endpoint.
    pub started_at: Instant,
}

impl AppState {
    /// Build the state from its parts.
    pub fn new(config: Arc<Config>, db: Db, git: GitStore) -> Self {
        Self {
            config,
            db,
            git,
            transport: SmartHttp::new(),
            started_at: Instant::now(),
        }
    }

    /// Use a transport that runs a specific git binary, with the environment git's hooks need.
    #[must_use]
    pub fn with_transport(mut self, transport: SmartHttp) -> Self {
        self.transport = transport;
        self
    }

    /// The clone URL a visitor is given, and the one the API reports.
    pub fn clone_url(&self, full_name: &str) -> String {
        format!("{}/{full_name}.git", self.config.origin())
    }

    /// The browsable URL for a repository.
    pub fn html_url(&self, path: &str) -> String {
        format!("{}/{}", self.config.origin(), path.trim_start_matches('/'))
    }
}

/// Largest JSON request body accepted, which is generous for an issue body and small for a URL.
const API_BODY_LIMIT: usize = 2 * 1024 * 1024;

/// Largest git request body accepted per request. A push of a large binary can exceed the API's
/// limit many times over, so this is the number to raise first when a push is refused; the process
/// on the other side of the pipe is git, which streams whatever it is given.
const GIT_BODY_LIMIT: usize = 4 * 1024 * 1024 * 1024;

/// Build the whole application router.
pub fn router(state: AppState) -> Router {
    let api = api::router()
        .fallback(api::not_found)
        .layer(DefaultBodyLimit::max(API_BODY_LIMIT))
        .layer(RequestBodyLimitLayer::new(API_BODY_LIMIT));

    // Git bodies are streams and are limited by their own layer, and compressing them would corrupt
    // the pkt-line framing.
    let git = git_http::router()
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(GIT_BODY_LIMIT));

    Router::new()
        .route("/healthz", get(api::meta::healthz))
        .nest("/api/v1", api)
        .merge(git)
        .fallback(assets::serve)
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(300),
        ))
        .layer(CatchPanicLayer::new())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(middleware::security_headers))
        .with_state(state)
}

/// Routes the API exposes, useful for a smoke test that the surface did not shrink.
pub const API_PREFIX: &str = "/api/v1";
