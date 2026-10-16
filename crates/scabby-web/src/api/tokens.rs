//! Personal access tokens and SSH keys.

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get};
use scabby_core::auth::ApiToken;
use scabby_db::tokens;
use serde::{Deserialize, Serialize};

use crate::auth::{CurrentUser, hash_token, new_api_token, ssh_fingerprint, token_prefix};
use crate::{AppState, WebError};

/// Token and key routes, all scoped to the caller.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/user/tokens", get(list_tokens).post(create_token))
        .route("/user/tokens/{id}", delete(delete_token))
        .route("/user/keys", get(list_keys).post(create_key))
        .route("/user/keys/{id}", delete(delete_key))
}

/// Body for creating a token.
#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    name: String,
    #[serde(default)]
    scopes: Option<Vec<String>>,
    #[serde(default)]
    expires_in_days: Option<i64>,
}

/// The one response that carries a token's secret.
#[derive(Debug, Serialize)]
pub struct CreatedToken {
    /// The stored token, without its secret.
    pub token: ApiToken,
    /// The secret. This is the only time the server can produce it, because only its hash is kept.
    pub plaintext: String,
}

/// `GET /api/v1/user/tokens`
pub async fn list_tokens(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<ApiToken>>, WebError> {
    Ok(Json(
        tokens::tokens_for_user(&state.db, user.0.id.get()).await?,
    ))
}

/// `POST /api/v1/user/tokens`
pub async fn create_token(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(request): Json<CreateTokenRequest>,
) -> Result<Response, WebError> {
    let name = request.name.trim();
    if name.is_empty() {
        return Err(WebError::validation("name", "must not be empty"));
    }

    let scopes = request
        .scopes
        .unwrap_or_else(|| vec!["repo".to_owned(), "user".to_owned()]);
    let expires_at = request
        .expires_in_days
        .map(|days| time::OffsetDateTime::now_utc() + time::Duration::days(days.clamp(1, 3650)));

    let secret = new_api_token();
    let token = tokens::create_token(
        &state.db,
        user.0.id.get(),
        name,
        &token_prefix(&secret),
        &hash_token(&secret),
        &scopes,
        expires_at,
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreatedToken {
            token,
            plaintext: secret,
        }),
    )
        .into_response())
}

/// `DELETE /api/v1/user/tokens/{id}`
pub async fn delete_token(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Response, WebError> {
    tokens::delete_token(&state.db, user.0.id.get(), id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

/// Body for adding a key.
#[derive(Debug, Deserialize)]
pub struct CreateKeyRequest {
    title: String,
    key: String,
}

/// `GET /api/v1/user/keys`
pub async fn list_keys(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Vec<scabby_core::auth::SshKey>>, WebError> {
    Ok(Json(
        tokens::ssh_keys_for_user(&state.db, user.0.id.get()).await?,
    ))
}

/// `POST /api/v1/user/keys`
pub async fn create_key(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(request): Json<CreateKeyRequest>,
) -> Result<Response, WebError> {
    let title = request.title.trim();
    if title.is_empty() {
        return Err(WebError::validation("title", "must not be empty"));
    }

    // The fingerprint is computed here rather than accepted from the client: it is what the instance
    // uses to recognise a key, and a client-supplied one could collide with someone else's.
    let key = request.key.trim();
    let fingerprint = ssh_fingerprint(key)?;
    let stored =
        tokens::create_ssh_key(&state.db, user.0.id.get(), title, key, &fingerprint).await?;

    Ok((StatusCode::CREATED, Json(stored)).into_response())
}

/// `DELETE /api/v1/user/keys/{id}`
pub async fn delete_key(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Response, WebError> {
    tokens::delete_ssh_key(&state.db, user.0.id.get(), id).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
