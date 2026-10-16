//! Git smart-HTTP: `owner/name.git/info/refs`, upload-pack, and receive-pack.
//!
//! A git client is told a repository exists at `/{owner}/{repo}.git`, so this is the one place the
//! server cannot present only JSON. The access check runs here, in Rust, before any bytes reach the
//! git process: an anonymous fetch of a public repository is allowed, a private repository answers
//! 401 so the client prompts, and a push without ownership never starts.

use std::path::PathBuf;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use futures_util::StreamExt as _;
use scabby_core::names::Slug;
use scabby_core::repo::Repository;
use scabby_core::user::User;
use scabby_db::repos;
use scabby_git::transport::Service;
use serde::Deserialize;
use tokio::io::AsyncWriteExt as _;
use tokio_util::io::ReaderStream;

use crate::auth::MaybeUser;
use crate::{AppState, WebError};

/// Routes a git client reaches.
pub fn router() -> Router<AppState> {
    Router::new()
        // The repository half of the path keeps its `.git` suffix, because a git client appends it
        // itself; the handler strips it before the slug is parsed.
        .route("/{owner}/{name}/info/refs", get(info_refs))
        .route("/{owner}/{name}/git-upload-pack", post(upload_pack))
        .route("/{owner}/{name}/git-receive-pack", post(receive_pack))
}

/// The `?service=` parameter git sends on the advertise request.
#[derive(Debug, Deserialize)]
pub struct AdvertiseQuery {
    service: Option<String>,
}

/// `GET /{owner}/{repo}.git/info/refs?service=git-upload-pack`
pub async fn info_refs(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    Query(query): Query<AdvertiseQuery>,
    headers: HeaderMap,
    user: MaybeUser,
) -> Result<Response, WebError> {
    // Without a service parameter the client is asking for the dumb protocol, which Scabby does not
    // serve. Answering 404 is what a smart-only host is supposed to do.
    let service = query
        .service
        .as_deref()
        .and_then(Service::parse)
        .ok_or_else(|| {
            WebError::BadRequest("service must be git-upload-pack or git-receive-pack".to_owned())
        })?;

    let access = authorize(&state, &owner, &name, service, user.0.as_ref()).await?;
    let protocol = git_protocol(&headers);

    let body = state
        .transport
        .advertise(&access.directory, service, protocol.as_deref())
        .await?;

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, service.advertisement_content_type()),
            (header::CACHE_CONTROL, "no-cache".to_owned()),
        ],
        Body::from(body),
    )
        .into_response())
}

/// `POST /{owner}/{repo}.git/git-upload-pack`
pub async fn upload_pack(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    headers: HeaderMap,
    user: MaybeUser,
    request: Request,
) -> Result<Response, WebError> {
    serve_rpc(
        state,
        owner,
        name,
        Service::UploadPack,
        headers,
        user,
        request,
    )
    .await
}

/// `POST /{owner}/{repo}.git/git-receive-pack`
pub async fn receive_pack(
    State(state): State<AppState>,
    Path((owner, name)): Path<(String, String)>,
    headers: HeaderMap,
    user: MaybeUser,
    request: Request,
) -> Result<Response, WebError> {
    serve_rpc(
        state,
        owner,
        name,
        Service::ReceivePack,
        headers,
        user,
        request,
    )
    .await
}

/// Start the service process and hand its stdout back as a streaming response body.
async fn serve_rpc(
    state: AppState,
    owner: String,
    name: String,
    service: Service,
    headers: HeaderMap,
    user: MaybeUser,
    request: Request,
) -> Result<Response, WebError> {
    let access = authorize(&state, &owner, &name, service, user.0.as_ref()).await?;
    let protocol = git_protocol(&headers);
    let actor = access.user.as_ref().map(|user| user.username.to_string());

    let mut session = state.transport.start(
        &access.directory,
        service,
        protocol.as_deref(),
        actor.as_deref(),
    )?;
    let mut stdin = session
        .take_stdin()
        .ok_or_else(|| WebError::BadRequest("git did not open a stdin pipe".to_owned()))?;
    let stdout = session
        .take_stdout()
        .ok_or_else(|| WebError::BadRequest("git did not open a stdout pipe".to_owned()))?;

    tracing::debug!(
        repository = %access.repository.full_name(),
        service = service.name(),
        "serving a git request"
    );

    // git clients compress the request body, and only the compressed path needs the body in memory:
    // everything else is pumped straight through, which is what keeps a large push off the heap.
    let compressed = is_gzip(&headers);
    let body = request.into_body();

    if compressed {
        let raw = axum::body::to_bytes(body, GIT_INFLATE_LIMIT)
            .await
            .map_err(|error| WebError::BadRequest(format!("request body: {error}")))?;
        let decoded = inflate(&raw)?;
        tokio::spawn(async move {
            if let Err(error) = stdin.write_all(&decoded).await {
                tracing::warn!(%error, "writing a decompressed request body to git failed");
            }
            let _ = stdin.shutdown().await;
        });
    } else {
        tokio::spawn(async move {
            let mut stream = body.into_data_stream();
            while let Some(chunk) = stream.next().await {
                match chunk {
                    Ok(chunk) => {
                        if let Err(error) = stdin.write_all(&chunk).await {
                            tracing::warn!(%error, "writing a request body to git failed");
                            break;
                        }
                    }
                    Err(error) => {
                        tracing::warn!(%error, "reading a request body failed");
                        break;
                    }
                }
            }
            // Closing stdin is what tells git the request is complete.
            let _ = stdin.shutdown().await;
        });
    }

    // The process is reaped on its own task, so its exit status reaches the log without holding up
    // the response, and the streaming body keeps stdout alive until the client has all of it.
    tokio::spawn(async move {
        if let Err(error) = session.finish().await {
            tracing::warn!(%error, "git service exited with a failure");
        }
    });

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, service.result_content_type()),
            (header::CACHE_CONTROL, "no-cache".to_owned()),
        ],
        Body::from_stream(ReaderStream::new(stdout)),
    )
        .into_response())
}

/// Largest compressed request body that will be decompressed in memory.
const GIT_INFLATE_LIMIT: usize = 64 * 1024 * 1024;

/// Whether the request body is gzip encoded, which git does for small upload-pack requests.
fn is_gzip(headers: &HeaderMap) -> bool {
    headers
        .get(header::CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("gzip"))
}

/// Decompress a gzip request body.
fn inflate(raw: &[u8]) -> Result<Vec<u8>, WebError> {
    use std::io::Read as _;

    let mut decoder = flate2::read::GzDecoder::new(raw);
    let mut decoded = Vec::new();
    decoder
        .read_to_end(&mut decoded)
        .map_err(|error| WebError::BadRequest(format!("gzip body: {error}")))?;
    Ok(decoded)
}

/// The `Git-Protocol` header, which asks for protocol v2.
fn git_protocol(headers: &HeaderMap) -> Option<String> {
    headers
        .get("git-protocol")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

/// A repository the caller may act on, and the directory git should run in.
pub struct Access {
    /// The repository row.
    pub repository: Repository,
    /// The caller, when there is one.
    pub user: Option<User>,
    /// Directory of the bare repository.
    pub directory: PathBuf,
}

/// Decide whether a caller may fetch from or push to a repository.
///
/// A private repository answers 401 to an anonymous client rather than 404, which is the one place
/// Scabby admits a repository exists without letting anyone read it: a git client needs the
/// challenge to know it should ask for a password.
async fn authorize(
    state: &AppState,
    owner: &str,
    name: &str,
    service: Service,
    user: Option<&User>,
) -> Result<Access, WebError> {
    let Some(name) = name.strip_suffix(".git") else {
        return Err(WebError::Domain(scabby_core::Error::not_found(
            "repository",
            format!("{owner}/{name}"),
        )));
    };

    let slug = Slug::parse(&format!("{owner}/{name}"))?;
    let repository = repos::by_slug(&state.db, &slug).await?;
    let is_owner = user.is_some_and(|user| user.id == repository.owner.id);

    if repository.visibility.is_private() && !is_owner {
        return match user {
            Some(_) => Err(WebError::Domain(scabby_core::Error::not_found(
                "repository",
                slug.full_name(),
            ))),
            None => Err(WebError::unauthorized(true)),
        };
    }

    // Fetching is fine for anyone who can read it; pushing is the owner's alone.
    if service == Service::ReceivePack && !is_owner {
        return match user {
            Some(_) => Err(WebError::forbidden("only the repository owner may push")),
            None => Err(WebError::unauthorized(true)),
        };
    }

    let directory = state.git.path(&slug);
    if !state.git.exists(&slug) {
        // The row exists and the directory does not, which is a broken instance rather than a
        // missing repository, so it is reported as such instead of as a 404 the user could fix.
        return Err(WebError::Domain(scabby_core::Error::internal(format!(
            "repository {} has no directory on disk",
            slug.full_name()
        ))));
    }

    Ok(Access {
        repository,
        user: user.cloned(),
        directory,
    })
}
