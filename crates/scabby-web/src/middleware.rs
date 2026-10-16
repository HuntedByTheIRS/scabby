//! Small response layers.

use axum::extract::{Request, State};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

use crate::AppState;
use crate::assets;

/// Headers every response carries.
///
/// Scripts and styles come from this origin, images may be remote because a user's avatar can be, and
/// nothing may be framed. The one script that is not a file is the boot script SvelteKit writes into
/// the shell, which [`assets::content_security_policy`] hashes into the policy.
pub async fn security_headers(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        axum::http::header::CONTENT_SECURITY_POLICY,
        assets::content_security_policy(state.config.frontend_dir.as_deref()),
    );
    headers.insert(
        axum::http::header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        axum::http::header::REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );

    response
}
