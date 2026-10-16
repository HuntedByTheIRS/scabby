//! REST API version 1.
//!
//! The shape follows GitHub's: snake_case fields, resources nested under `/repos/{owner}/{repo}`,
//! `Link` headers for pagination, and a `message` on every error. The differences from GitHub are
//! listed in `docs/api.md`, and the one that shows up in every list response is the envelope:
//! collections carry `items`, `page`, `per_page`, `total`, and `total_pages` instead of a bare array.
//! A path under the prefix that matches nothing is a JSON 404, never the single-page app.

pub mod auth;
pub mod events;
pub mod issues;
pub mod meta;
pub mod repos;
pub mod tokens;
pub mod users;

use axum::Json;
use axum::Router;
use axum::extract::OriginalUri;
use axum::http::{HeaderValue, Method, Uri, header};
use axum::response::{IntoResponse, Response};
use scabby_core::pagination::{Page, PageResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AppState;
use crate::error::WebError;

/// Every route under `/api/v1`.
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(meta::router())
        .merge(auth::router())
        .merge(users::router())
        .merge(tokens::router())
        .merge(repos::router())
        .merge(issues::router())
        .merge(events::router())
}

/// The `?page=&per_page=` pair every collection accepts.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PageQuery {
    /// One-based page number.
    pub page: Option<u32>,
    /// Items per page.
    pub per_page: Option<u32>,
}

impl PageQuery {
    /// Clamped page, with the defaults applied.
    pub fn page(self) -> Page {
        Page::new(self.page, self.per_page)
    }
}

/// A paginated collection response, with the `Link` header GitHub clients look for.
pub(crate) fn page_response<T: Serialize>(
    state: &AppState,
    uri: &Uri,
    items: Vec<T>,
    page: Page,
    total: i64,
) -> Response {
    let body = PageResult::new(items, page, total);
    // Read before the body is moved into the response.
    let total_pages = body.total_pages;
    let mut response = Json(body).into_response();

    // `total` is always known here: this is the counted path, and an uncounted list builds its own
    // header from `Page::link_header_without_total`.
    if let Some(total_pages) = total_pages
        && let Some(link) = page.link_header(&page_url(state, uri), total_pages)
        && let Ok(value) = HeaderValue::from_str(&link)
    {
        response.headers_mut().insert(header::LINK, value);
    }
    response
}

/// The request URL with any `page` parameter removed, which is what the `Link` header is built on.
pub(crate) fn page_url(state: &AppState, uri: &Uri) -> String {
    let query = uri.query().unwrap_or_default();
    let kept: Vec<&str> = query
        .split('&')
        .filter(|part| !part.is_empty() && !part.starts_with("page="))
        .collect();

    let path = uri.path();
    if kept.is_empty() {
        format!("{}{path}", state.config.origin())
    } else {
        format!("{}{path}?{}", state.config.origin(), kept.join("&"))
    }
}

/// A patch body, where every field is optional and an absent field means "leave it alone".
///
/// Deserializing into `Value` first is what makes "absent" distinguishable from "null": a JSON body
/// that omits a key and one that sets it to `null` both arrive here as `None`, which is the
/// behaviour an API client expects from a PATCH.
pub(crate) fn field<T: serde::de::DeserializeOwned>(
    body: &Value,
    name: &str,
) -> Result<Option<T>, crate::WebError> {
    match body.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => serde_json::from_value(value.clone())
            .map(Some)
            .map_err(|error| crate::WebError::validation("body", format!("{name}: {error}"))),
    }
}

/// Answer for a path under `/api/v1` that matches no route.
///
/// Without this the request falls through to the single-page app, which answers `200` with an HTML
/// shell: a client that mistyped an endpoint would see success and then fail parsing JSON.
pub async fn not_found(method: Method, OriginalUri(uri): OriginalUri) -> WebError {
    WebError::not_found("endpoint", format!("{method} {uri}"))
}
