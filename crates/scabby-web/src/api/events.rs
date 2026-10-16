//! The activity feed behind the dashboard.

use axum::Router;
use axum::extract::{Query, State};
use axum::http::Uri;
use axum::response::Response;
use axum::routing::get;
use scabby_db::events;
use serde::Deserialize;

use crate::api::{PageQuery, page_response};
use crate::auth::CurrentUser;
use crate::{AppState, WebError};

/// Feed routes.
pub fn router() -> Router<AppState> {
    Router::new().route("/events", get(recent))
}

/// Query for the feed.
#[derive(Debug, Deserialize)]
pub struct EventListQuery {
    /// Page number.
    pub page: Option<u32>,
    /// Items per page.
    pub per_page: Option<u32>,
}

/// `GET /api/v1/events`
///
/// The feed is instance-wide, which is why it needs credentials: an entry names a repository, and a
/// signed-out visitor is not entitled to the list of repositories that exist. Once a caller is
/// signed in they see the whole feed, including movements in private repositories they cannot open,
/// because the alternative is a per-row visibility lookup on every page of the feed. That is a known
/// gap in v0, recorded here rather than discovered later.
pub async fn recent(
    State(state): State<AppState>,
    Query(query): Query<EventListQuery>,
    uri: Uri,
    user: CurrentUser,
) -> Result<Response, WebError> {
    let _ = user;
    let page = PageQuery {
        page: query.page,
        per_page: query.per_page,
    }
    .page();
    let (found, total) = events::recent(&state.db, page).await?;

    Ok(page_response(&state, &uri, found, page, total))
}
