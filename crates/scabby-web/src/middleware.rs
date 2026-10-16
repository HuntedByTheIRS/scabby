//! Small response layers.

use axum::extract::Request;
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;

/// Headers every response carries.
///
/// The UI is a compiled bundle with no inline script of its own, so the content security policy can
/// be strict: scripts and styles come from this origin, images may be remote because a user's avatar
/// can be, and nothing may be framed.
pub async fn security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        axum::http::header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; img-src 'self' data: https:; style-src 'self' 'unsafe-inline'; \
             script-src 'self'; connect-src 'self'; form-action 'self'; frame-ancestors 'none'; \
             base-uri 'self'",
        ),
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
