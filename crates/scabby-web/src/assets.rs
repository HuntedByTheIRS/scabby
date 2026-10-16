//! Serving the user interface: the embedded bundle, or a directory when one is given.

use std::path::Path;
use std::sync::OnceLock;

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderValue, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use rust_embed::RustEmbed;
use sha2::{Digest, Sha256};

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

/// Largest shell accepted when hashing its inline scripts.
const SHELL_LIMIT: usize = 4 * 1024 * 1024;

/// The policy directives every response carries, before the script hashes are appended.
const POLICY_HEAD: &str = "default-src 'self'; img-src 'self' data: https:; style-src 'self' \
                            'unsafe-inline'; script-src 'self'";

/// The rest of the policy, after the script hashes.
const POLICY_TAIL: &str = "; connect-src 'self'; form-action 'self'; frame-ancestors 'none'; \
                           base-uri 'self'";

/// The content security policy this shell needs.
///
/// SvelteKit's static adapter boots the single-page app from a script written inline into
/// `index.html`. A policy of `script-src 'self'` alone blocks that script, so the shell loads, the
/// app never starts, and the browser shows an empty page. Naming the hash of each inline script
/// allows exactly this shell without `'unsafe-inline'`, which would allow any injected script.
///
/// The embedded bundle cannot change while the process runs, so its policy is computed once. A
/// directory given with `--frontend-dir` can change between requests, so it is hashed per request.
pub fn content_security_policy(frontend_dir: Option<&Path>) -> HeaderValue {
    static EMBEDDED: OnceLock<HeaderValue> = OnceLock::new();

    match frontend_dir {
        None => EMBEDDED
            .get_or_init(|| policy_for(embedded_shell().as_deref()))
            .clone(),
        Some(dir) => {
            let shell = std::fs::read(dir.join("index.html"))
                .ok()
                .filter(|bytes| bytes.len() <= SHELL_LIMIT);
            policy_for(shell.as_deref())
        }
    }
}

/// The embedded shell, if this build has one.
fn embedded_shell() -> Option<Vec<u8>> {
    let shell = Embedded::get("index.html")?;
    let bytes = shell.data.into_owned();
    (bytes.len() <= SHELL_LIMIT).then_some(bytes)
}

/// Build the policy, naming the hash of every inline script in `shell`.
///
/// Without a shell there is nothing inline to allow, and the strict policy stands on its own.
fn policy_for(shell: Option<&[u8]>) -> HeaderValue {
    let mut policy = String::from(POLICY_HEAD);
    if let Some(shell) = shell {
        for hash in inline_script_hashes(shell) {
            policy.push_str(" 'sha256-");
            policy.push_str(&hash);
            policy.push('\'');
        }
    }
    policy.push_str(POLICY_TAIL);

    HeaderValue::from_str(&policy).unwrap_or_else(|error| {
        // The policy is built from ASCII constants and base64, so this cannot happen; if it ever
        // does, the strict policy without hashes is the safe thing to send.
        tracing::error!(%error, "the content security policy was not a valid header value");
        HeaderValue::from_static(POLICY_HEAD)
    })
}

/// The base64 SHA-256 of every inline script body in the shell, which is how CSP names one.
fn inline_script_hashes(shell: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(shell);
    let mut hashes = Vec::new();
    let mut rest = text.as_ref();

    while let Some(open) = rest.find("<script") {
        let after_open = &rest[open + "<script".len()..];
        let Some(attributes) = after_open.find('>') else {
            break;
        };
        let after_attributes = &after_open[attributes + 1..];
        let Some(close) = after_attributes.find("</script>") else {
            break;
        };

        // An empty body is a script that loads from `src`, which the policy already allows.
        let body = &after_attributes[..close];
        if !body.trim().is_empty() {
            let digest = Sha256::digest(body.as_bytes());
            hashes.push(base64::engine::general_purpose::STANDARD.encode(digest));
        }

        rest = &after_attributes[close + "</script>".len()..];
    }

    hashes
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    /// What a browser does with the boot script is the difference between an application and a blank
    /// page, so the hash in the policy is the value the browser will compute.
    #[test]
    fn the_policy_names_the_hash_of_the_inline_script() {
        let body = "\n  { window.__sveltekit = { base: \"\" }; }\n";
        let shell = format!("<html><head><script>{body}</script></head></html>");

        let digest = Sha256::digest(body.as_bytes());
        let expected = base64::engine::general_purpose::STANDARD.encode(digest);

        let hashes = inline_script_hashes(shell.as_bytes());
        assert_eq!(hashes, vec![expected.clone()]);

        let policy = policy_for(Some(shell.as_bytes()));
        let policy = policy.to_str().expect("the policy is ASCII");
        assert!(
            policy.contains(&format!("'sha256-{expected}'")),
            "the policy has to carry the hash: {policy}"
        );
        let script_src = policy.split("script-src").nth(1).unwrap_or_default();
        assert!(
            !script_src.contains("unsafe-inline"),
            "script-src must not fall back to unsafe-inline: {script_src}"
        );
        assert!(script_src.starts_with(" 'self' 'sha256-"));
    }

    #[test]
    fn a_shell_with_no_inline_script_keeps_the_strict_policy() {
        let shell = b"<html><head><script src=\"/app.js\"></script></head></html>";
        assert!(inline_script_hashes(shell).is_empty());

        let policy = policy_for(Some(shell));
        assert_eq!(policy.to_str().unwrap().matches("'sha256-").count(), 0);
    }

    #[test]
    fn every_inline_script_gets_its_own_hash() {
        let shell = b"<script>one()</script><script src=\"a.js\"></script><script>two()</script>";
        assert_eq!(inline_script_hashes(shell).len(), 2);
    }
}
