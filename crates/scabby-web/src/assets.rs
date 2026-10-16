//! Serving the user interface: the embedded bundle, or a directory when one is given.

use axum::body::Body;
use axum::extract::State;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

use crate::AppState;

/// The Svelte build output, embedded at compile time.
///
/// `web/build` is produced by `npm run build` in `web/`. The build script writes a placeholder page
/// when it is missing, so a fresh checkout compiles and tells the visitor what to run.
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/../../web/build/"]
struct Embedded;

/// Serve a static asset, falling back to the single-page application shell.
///
/// A path that looks like a file and is not there is a 404; anything else is a route the SPA owns,
/// so it gets `index.html` and the client router takes over.
pub async fn serve(State(state): State<AppState>, uri: Uri) -> Response {
    let requested = uri.path().trim_start_matches('/');
    let requested = if requested.is_empty() {
        "index.html"
    } else {
        requested
    };

    // A path that climbs out of the directory is refused before it is joined to anything.
    if requested
        .split('/')
        .any(|segment| segment == ".." || segment == ".")
    {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }

    if let Some(dir) = state.config.frontend_dir.clone() {
        return from_directory(&dir, requested).await;
    }

    match Embedded::get(requested) {
        Some(file) => asset_response(requested, file.data.into_owned()),
        None if looks_like_a_file(requested) => {
            (StatusCode::NOT_FOUND, "not found").into_response()
        }
        None => match Embedded::get("index.html") {
            Some(index) => asset_response("index.html", index.data.into_owned()),
            None => (
                StatusCode::NOT_FOUND,
                "the user interface is not part of this build",
            )
                .into_response(),
        },
    }
}

/// Serve from a directory instead of the embedded copy, which is what `--frontend-dir` does.
async fn from_directory(dir: &std::path::Path, requested: &str) -> Response {
    let path = dir.join(requested);
    if let Ok(contents) = tokio::fs::read(&path).await {
        return asset_response(requested, contents);
    }

    if looks_like_a_file(requested) {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }

    match tokio::fs::read(dir.join("index.html")).await {
        Ok(index) => asset_response("index.html", index),
        Err(_) => (
            StatusCode::NOT_FOUND,
            format!("{} has no index.html", dir.display()),
        )
            .into_response(),
    }
}

/// Content type and cache policy for one file.
fn asset_response(path: &str, contents: Vec<u8>) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();

    // The shell must be revalidated on every load, or a deploy leaves clients on an old bundle that
    // asks for assets the server no longer has. A fingerprinted asset never changes, so it is cached
    // for a year.
    let cache = if path == "index.html" {
        "no-cache"
    } else if is_fingerprinted(path) {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    };

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime.as_ref().to_owned()),
            (header::CACHE_CONTROL, cache.to_owned()),
        ],
        Body::from(contents),
    )
        .into_response()
}

/// Whether a path carries a content hash, which makes it safe to cache forever.
///
/// Vite writes hashed assets under a directory named `immutable`, and names them `app.a1b2c3d4.js`;
/// both shapes are recognised so a custom frontend that keeps the hashes still caches well.
fn is_fingerprinted(path: &str) -> bool {
    if path.contains("/immutable/") {
        return true;
    }

    // `name.hash.ext`: the segment before the extension is the hash vite wrote.
    let file = path.rsplit('/').next().unwrap_or(path);
    let segments: Vec<&str> = file.split('.').collect();
    if segments.len() < 3 {
        return false;
    }

    let hash = segments[segments.len() - 2];
    hash.len() >= 8 && hash.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Whether the request is for something that should exist as a file rather than a client route.
fn looks_like_a_file(path: &str) -> bool {
    let last = path.rsplit('/').next().unwrap_or(path);
    last.contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_hashed_asset_names() {
        assert!(is_fingerprinted("_app/immutable/entry/start.a1b2c3d4.js"));
        assert!(is_fingerprinted("_app/immutable/chunks/Button.a1b2c3d4.js"));
        assert!(is_fingerprinted("assets/app.9f8e7d6c.css"));
        assert!(!is_fingerprinted("index.html"));
        assert!(
            !is_fingerprinted("favicon.ico"),
            "an unhashed icon must revalidate"
        );
    }

    #[test]
    fn client_routes_are_not_mistaken_for_files() {
        assert!(!looks_like_a_file("ada/scabby/commits"));
        assert!(looks_like_a_file("favicon.ico"));
        assert!(looks_like_a_file("_app/immutable/app.js"));
    }
}
