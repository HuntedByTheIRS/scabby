//! Registration, login, logout, and the session probe.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use scabby_core::event::EventKind;
use scabby_core::names::Name;
use scabby_core::user::User;
use scabby_db::{DbError, events, tokens, users};
use serde::{Deserialize, Serialize};

use crate::auth::{
    CurrentUser, MIN_PASSWORD_LEN, clear_cookie, hash_password, hash_token, new_session_token,
    session_cookie, session_expiry, verify_password,
};
use crate::{AppState, WebError};

/// Session routes. GitHub has no login endpoint to copy, so these are Scabby's own.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/session", get(session))
}

/// Registration body.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    username: String,
    email: String,
    password: String,
}

/// Login body.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    username_or_email: String,
    password: String,
}

/// The user object as the API returns it.
#[derive(Debug, Serialize)]
pub struct SessionBody {
    /// The signed-in account.
    pub user: User,
}

/// `POST /api/v1/auth/register`
///
/// The first account on an empty instance becomes an administrator, because otherwise a fresh
/// deployment has no way to reach the admin commands except the command line it may not have.
pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<Response, WebError> {
    if !state.config.registration.is_open() {
        return Err(WebError::forbidden(
            "registration is closed on this instance",
        ));
    }

    let username = Name::parse(&request.username)?;
    let email = request.email.trim().to_owned();

    if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        return Err(WebError::validation("email", "must look like an address"));
    }
    if request.password.chars().count() < MIN_PASSWORD_LEN {
        return Err(WebError::validation(
            "password",
            format!("must be at least {MIN_PASSWORD_LEN} characters"),
        ));
    }

    let password_hash = hash_password(&request.password)?;
    let is_admin = users::count(&state.db).await? == 0;
    let user = users::create(&state.db, &username, &email, &password_hash, is_admin).await?;

    events::record(
        &state.db,
        Some(user.id.get()),
        None,
        EventKind::UserRegistered,
    )
    .await?;

    let cookie = start_session(&state, &user).await?;
    Ok(with_cookie(Json(SessionBody { user }), cookie))
}

/// `POST /api/v1/auth/login`
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Response, WebError> {
    let credentials = match users::credentials(&state.db, request.username_or_email.trim()).await {
        Ok(credentials) => credentials,
        // A missing account and a wrong password answer the same way, so the endpoint cannot be used
        // to find out which handles exist.
        Err(DbError::NotFound { .. }) => return Err(WebError::unauthorized(false)),
        Err(error) => return Err(error.into()),
    };

    if !verify_password(&credentials.password_hash, &request.password) {
        return Err(WebError::unauthorized(false));
    }

    let cookie = start_session(&state, &credentials.user).await?;
    Ok(with_cookie(
        Json(SessionBody {
            user: credentials.user,
        }),
        cookie,
    ))
}

/// `POST /api/v1/auth/logout`
pub async fn logout(State(state): State<AppState>, headers: axum::http::HeaderMap) -> Response {
    if let Some(token) = crate::auth::cookie_token(&headers) {
        // Revoking the row matters: clearing the cookie alone would leave the session usable for
        // anyone who had a copy of it.
        let _ = tokens::delete_session(&state.db, &hash_token(&token)).await;
    }

    let mut response = axum::http::StatusCode::NO_CONTENT.into_response();
    set_cookie(&mut response, clear_cookie(is_secure(&state)));
    response
}

/// `GET /api/v1/auth/session`
pub async fn session(user: CurrentUser) -> Json<SessionBody> {
    Json(SessionBody { user: user.0 })
}

/// Create a session row and return the cookie that carries it.
async fn start_session(state: &AppState, user: &User) -> Result<String, WebError> {
    let token = new_session_token();
    tokens::create_session(
        &state.db,
        user.id.get(),
        &hash_token(&token),
        None,
        session_expiry(),
    )
    .await?;
    Ok(session_cookie(&token, is_secure(state)))
}

/// Whether cookies should carry `Secure`, which follows the configured base URL.
fn is_secure(state: &AppState) -> bool {
    state.config.base_url.scheme() == "https"
}

/// Attach a session cookie to a response.
fn with_cookie(body: impl IntoResponse, cookie: String) -> Response {
    let mut response = body.into_response();
    set_cookie(&mut response, cookie);
    response
}

fn set_cookie(response: &mut Response, cookie: String) {
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().append(header::SET_COOKIE, value);
    }
}
