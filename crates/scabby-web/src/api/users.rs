//! The signed-in user and public profiles.

use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::Uri;
use axum::response::Response;
use axum::routing::get;
use scabby_core::user::User;
use scabby_db::{repos as repositories, users};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::repos::RepositoryDto;
use crate::api::{PageQuery, field, page_response};
use crate::auth::{CurrentUser, MaybeUser};
use crate::{AppState, WebError};

/// Routes for accounts.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/user", get(me).patch(update_me))
        .route("/users/{username}", get(profile))
        .route("/users/{username}/repos", get(user_repos))
}

/// `GET /api/v1/user`
pub async fn me(user: CurrentUser) -> Json<User> {
    Json(user.0)
}

/// `PATCH /api/v1/user`
pub async fn update_me(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(body): Json<Value>,
) -> Result<Json<User>, WebError> {
    let display_name: Option<String> = field(&body, "display_name")?;
    let bio: Option<String> = field(&body, "bio")?;

    let updated = users::update_profile(
        &state.db,
        user.0.id.get(),
        display_name.as_deref(),
        bio.as_deref(),
    )
    .await?;

    Ok(Json(updated))
}

/// `GET /api/v1/users/{username}`
pub async fn profile(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> Result<Json<User>, WebError> {
    // Public profiles never carry an email address, whoever is asking.
    let user = users::by_username(&state.db, &username, false).await?;
    Ok(Json(user))
}

/// Query for a repository list.
#[derive(Debug, Deserialize)]
pub struct RepoListQuery {
    /// Page number.
    pub page: Option<u32>,
    /// Items per page.
    pub per_page: Option<u32>,
    /// Whether to measure the git directories while listing them.
    #[serde(default)]
    pub stats: bool,
}

/// `GET /api/v1/users/{username}/repos`
pub async fn user_repos(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Query(query): Query<RepoListQuery>,
    uri: Uri,
    viewer: MaybeUser,
) -> Result<Response, WebError> {
    let owner = users::by_username(&state.db, &username, false).await?;
    let page = PageQuery {
        page: query.page,
        per_page: query.per_page,
    }
    .page();
    let viewer_id = viewer.0.as_ref().map(|user| user.id.get());

    let (found, total) =
        repositories::list_owned(&state.db, owner.id.get(), viewer_id, page).await?;
    let items = RepositoryDto::from_many(&state, found, query.stats);

    Ok(page_response(&state, &uri, items, page, total))
}

/// The `stats` flag on a list is a query on the request, so it travels with the DTOs.
#[derive(Debug, Clone, Serialize)]
pub struct _Unused;
