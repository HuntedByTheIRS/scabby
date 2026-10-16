//! Cookies, credential extraction, and the extractors handlers use.
//!
//! The hashing itself lives in `scabby-core::crypto`, because the command line writes the same
//! hashes the login endpoint verifies. This module deals only with HTTP: which header carries a
//! credential, what a session cookie looks like, and how a request's owner is resolved.

use axum::extract::FromRequestParts;
use axum::http::{HeaderMap, header};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use scabby_core::user::User;
use scabby_db::{Db, tokens};

use crate::AppState;
use crate::WebError;

/// Password hashing, token generation, and key fingerprints, re-exported so the rest of the web
/// layer has one import for credentials.
pub use scabby_core::crypto::{
    MIN_PASSWORD_LEN, hash_password, hash_token, new_api_token, new_session_token, ssh_fingerprint,
    token_prefix, verify_password,
};

/// Cookie holding the browser session.
pub const SESSION_COOKIE: &str = "scabby_session";

/// How long a browser session lasts.
pub const SESSION_TTL_DAYS: i64 = 30;

/// Session expiry as an offset timestamp.
pub fn session_expiry() -> time::OffsetDateTime {
    time::OffsetDateTime::now_utc() + time::Duration::days(SESSION_TTL_DAYS)
}

/// The `Set-Cookie` value that starts a session.
///
/// `Secure` follows the configured base URL: an instance behind TLS gets it, a plain development
/// instance does not, because a browser silently drops a Secure cookie on http and the developer
/// then cannot sign in at all.
pub fn session_cookie(token: &str, secure: bool) -> String {
    let mut cookie = format!(
        "{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        SESSION_TTL_DAYS * 24 * 60 * 60
    );
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

/// The `Set-Cookie` value that ends a session.
pub fn clear_cookie(secure: bool) -> String {
    let mut cookie = format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0");
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

/// The session token carried by a request's cookies, if any.
pub fn cookie_token(headers: &HeaderMap) -> Option<String> {
    let cookies = headers.get(header::COOKIE)?.to_str().ok()?;
    for cookie in cookies.split(';') {
        let cookie = cookie.trim();
        if let Some(value) = cookie.strip_prefix(&format!("{SESSION_COOKIE}=")) {
            let value = value.trim();
            if !value.is_empty() {
                return Some(value.to_owned());
            }
        }
    }
    None
}

/// A token presented by a client, and where it was found.
enum Presented {
    /// `Authorization: Bearer` or `Authorization: token`.
    Bearer(String),
    /// `Authorization: Basic`, which is how a git client sends a password.
    Basic(String),
    /// The session cookie.
    Cookie(String),
}

/// Read the credentials a request carries, if any.
fn presented(headers: &HeaderMap) -> Option<Presented> {
    if let Some(value) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        if let Some(token) = value
            .strip_prefix("Bearer ")
            .or_else(|| value.strip_prefix("token "))
        {
            return Some(Presented::Bearer(token.trim().to_owned()));
        }
        if let Some(encoded) = value.strip_prefix("Basic ") {
            // `user:password`, where the password is a personal access token. The username half is
            // free-form: git asks for one, and nothing checks it, so anything can be typed there.
            let decoded = STANDARD.decode(encoded.trim()).unwrap_or_default();
            let text = String::from_utf8_lossy(&decoded);
            if let Some((_, password)) = text.split_once(':') {
                return Some(Presented::Basic(password.to_owned()));
            }
        }
    }

    cookie_token(headers).map(Presented::Cookie)
}

/// Resolve the caller, if the request carries credentials that check out.
pub async fn resolve_user(db: &Db, headers: &HeaderMap) -> Result<Option<User>, WebError> {
    let Some(presented) = presented(headers) else {
        return Ok(None);
    };

    let user = match presented {
        Presented::Bearer(token) | Presented::Basic(token) => {
            tokens::token_user(db, &hash_token(&token)).await?
        }
        Presented::Cookie(token) => tokens::session_user(db, &hash_token(&token)).await?,
    };

    Ok(user)
}

/// The caller, when there is one. Handlers that work for anonymous visitors take this.
pub struct MaybeUser(pub Option<User>);

impl FromRequestParts<AppState> for MaybeUser {
    type Rejection = WebError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(resolve_user(&state.db, &parts.headers).await?))
    }
}

/// The caller, or a 401. Handlers that need an account take this.
pub struct CurrentUser(pub User);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = WebError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match resolve_user(&state.db, &parts.headers).await? {
            Some(user) => Ok(Self(user)),
            // No challenge here: the browser UI shows a sign-in page, and a `WWW-Authenticate`
            // header would make it raise a password dialog instead.
            None => Err(WebError::unauthorized(false)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookies_round_trip() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "other=1; scabby_session=abc123; another=2".parse().unwrap(),
        );
        assert_eq!(cookie_token(&headers).as_deref(), Some("abc123"));

        let cookie = session_cookie("abc123", false);
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(!cookie.contains("Secure"));
        assert!(session_cookie("abc123", true).contains("Secure"));
        assert!(clear_cookie(false).contains("Max-Age=0"));

        headers.insert(header::COOKIE, "nothing=here".parse().unwrap());
        assert!(cookie_token(&headers).is_none());
    }

    #[test]
    fn credentials_are_read_from_all_three_headers() {
        let mut headers = HeaderMap::new();
        assert!(presented(&headers).is_none());

        headers.insert(header::AUTHORIZATION, "Bearer abc".parse().unwrap());
        assert!(matches!(presented(&headers), Some(Presented::Bearer(token)) if token == "abc"));

        headers.insert(header::AUTHORIZATION, "token def".parse().unwrap());
        assert!(matches!(presented(&headers), Some(Presented::Bearer(token)) if token == "def"));

        // `git clone http://user:scabby_pat_...@host/...`
        let encoded = STANDARD.encode("ada:scabby_pat_secret");
        headers.insert(
            header::AUTHORIZATION,
            format!("Basic {encoded}").parse().unwrap(),
        );
        assert!(
            matches!(presented(&headers), Some(Presented::Basic(token)) if token == "scabby_pat_secret")
        );

        // A malformed Basic value is not a credential, and must not panic the request.
        headers.insert(
            header::AUTHORIZATION,
            "Basic !!!not-base64!!!".parse().unwrap(),
        );
        assert!(presented(&headers).is_none());
    }
}
