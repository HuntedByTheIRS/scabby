//! Repositories, their git reads, and the routes that read a repository's history.

use axum::Json;
use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::Uri;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use scabby_core::Error;
use scabby_core::event::EventKind;
use scabby_core::git::{Blob, Branch, CommitDetail, CommitSummary, GitStats, Tag, TreeEntry};
use scabby_core::names::{Name, Slug};
use scabby_core::repo::{Repository, Visibility};
use scabby_db::{events, repos as repositories};
use scabby_git::read;
use scabby_git::{GitError, diff};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{PageQuery, field, page_response};
use crate::auth::{CurrentUser, MaybeUser};
use crate::{AppState, WebError};

/// Branch a new repository starts on.
const DEFAULT_BRANCH: &str = "main";

/// How far back a commit count is walked before Scabby stops counting and says so.
const COMMIT_COUNT_CAP: usize = 100_000;

/// Repository routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/repos", get(list_repos))
        .route("/user/repos", post(create_repo))
        .route(
            "/repos/{owner}/{repo}",
            get(get_repo).patch(patch_repo).delete(delete_repo),
        )
        .route("/repos/{owner}/{repo}/branches", get(branches))
        .route("/repos/{owner}/{repo}/tags", get(tags))
        .route("/repos/{owner}/{repo}/stats", get(stats))
        .route("/repos/{owner}/{repo}/commits", get(commits))
        .route("/repos/{owner}/{repo}/commits/{sha}", get(commit))
        .route("/repos/{owner}/{repo}/contents", get(contents_root))
        .route("/repos/{owner}/{repo}/contents/{*path}", get(contents))
        .route("/repos/{owner}/{repo}/raw/{revision}/{*path}", get(raw))
        .route("/repos/{owner}/{repo}/readme", get(readme))
}

/// A repository as the API returns it: the stored fields, the two URLs a client would otherwise
/// have to build itself, and the on-disk measurements when they were asked for.
#[derive(Debug, Serialize)]
pub struct RepositoryDto {
    #[serde(flatten)]
    repository: Repository,
    full_name: String,
    clone_url: String,
    html_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    git: Option<GitStats>,
}

impl RepositoryDto {
    /// Wrap one repository.
    pub fn new(state: &AppState, repository: Repository) -> Self {
        let full_name = repository.full_name();
        Self {
            clone_url: state.clone_url(&full_name),
            html_url: state.html_url(&full_name),
            full_name,
            git: None,
            repository,
        }
    }

    /// Attach the on-disk measurements.
    fn with_stats(mut self, git: GitStats) -> Self {
        self.git = Some(git);
        self
    }

    /// Wrap a page of repositories, measuring them on the rayon pool when asked.
    ///
    /// Measuring opens every repository, so it is opt-in per request: a plain list is a single
    /// query, while `?stats=true` spends CPU on the page's worth of directories in parallel.
    pub fn from_many(
        state: &AppState,
        repositories: Vec<Repository>,
        with_stats: bool,
    ) -> Vec<Self> {
        if !with_stats {
            return repositories
                .into_iter()
                .map(|repository| Self::new(state, repository))
                .collect();
        }

        let targets: Vec<(Slug, String)> = repositories
            .iter()
            .map(|repository| (repository.slug(), repository.default_branch.clone()))
            .collect();

        repositories
            .into_iter()
            .zip(state.git.stats_all(&targets))
            .map(|(repository, stats)| match stats {
                Ok(stats) => Self::new(state, repository).with_stats(stats),
                Err(error) => {
                    // A missing or unreadable directory is a broken instance rather than a bad
                    // request, and it must not take the rest of the page down with it.
                    tracing::warn!(
                        repository = %repository.full_name(),
                        %error,
                        "could not measure a repository"
                    );
                    Self::new(state, repository)
                }
            })
            .collect()
    }
}

/// Query for the repository lists.
#[derive(Debug, Deserialize)]
pub struct RepoListQuery {
    /// Page number.
    pub page: Option<u32>,
    /// Items per page.
    pub per_page: Option<u32>,
    /// Measure each repository on disk as well.
    #[serde(default)]
    pub stats: bool,
}

/// `GET /api/v1/repos`
pub async fn list_repos(
    State(state): State<AppState>,
    Query(query): Query<RepoListQuery>,
    uri: Uri,
    viewer: MaybeUser,
) -> Result<Response, WebError> {
    let page = PageQuery {
        page: query.page,
        per_page: query.per_page,
    }
    .page();
    let viewer_id = viewer.0.as_ref().map(|user| user.id.get());
    let (found, total) = repositories::list_visible(&state.db, viewer_id, page).await?;

    Ok(page_response(
        &state,
        &uri,
        RepositoryDto::from_many(&state, found, query.stats),
        page,
        total,
    ))
}

/// Body for creating a repository.
#[derive(Debug, Deserialize)]
pub struct CreateRepoRequest {
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    private: bool,
}

/// `POST /api/v1/user/repos`
pub async fn create_repo(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(request): Json<CreateRepoRequest>,
) -> Result<Response, WebError> {
    let name = Name::parse(&request.name)?;
    let visibility = if request.private {
        Visibility::Private
    } else {
        Visibility::Public
    };
    let description = request
        .description
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());

    let repository = repositories::create(
        &state.db,
        user.0.id.get(),
        &name,
        description,
        visibility,
        DEFAULT_BRANCH,
    )
    .await?;

    let slug = repository.slug();
    if let Err(error) = state.git.create(&slug, DEFAULT_BRANCH) {
        // A row without a directory answers every read with a 500, which is worse than no row at
        // all, so the insert is rolled back rather than left behind for someone else to find.
        if let Err(rollback) = repositories::delete(&state.db, repository.id.get()).await {
            tracing::error!(
                repository = %slug.full_name(),
                %rollback,
                "could not remove the row for a repository whose directory failed to be created"
            );
        }
        return Err(error.into());
    }

    let _ = events::record(
        &state.db,
        Some(user.0.id.get()),
        Some(repository.id.get()),
        EventKind::RepositoryCreated,
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(RepositoryDto::new(&state, repository)),
    )
        .into_response())
}

/// `GET /api/v1/repos/{owner}/{repo}`
pub async fn get_repo(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    viewer: MaybeUser,
) -> Result<Json<RepositoryDto>, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    Ok(Json(measure(&state, repository)))
}

/// `PATCH /api/v1/repos/{owner}/{repo}`
pub async fn patch_repo(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    user: CurrentUser,
    Json(body): Json<Value>,
) -> Result<Json<RepositoryDto>, WebError> {
    let repository = visible(&state, &owner, &repo, Some(&user.0)).await?;
    owner_only(&repository, &user.0)?;

    let description: Option<String> = field(&body, "description")?;
    let private: Option<bool> = field(&body, "private")?;
    let default_branch: Option<String> = field(&body, "default_branch")?;

    if let Some(branch) = &default_branch {
        // Pointing the default branch at a branch that is not there would break the overview page
        // and every clone URL built from it, so it is checked while the repository still works.
        ensure_branch_exists(&state, &repository, branch)?;
    }

    let patch = repositories::RepositoryPatch {
        description,
        visibility: private.map(|private| {
            if private {
                Visibility::Private
            } else {
                Visibility::Public
            }
        }),
        default_branch,
    };

    let updated = repositories::update(&state.db, repository.id.get(), &patch).await?;
    Ok(Json(measure(&state, updated)))
}

/// `DELETE /api/v1/repos/{owner}/{repo}`
pub async fn delete_repo(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    user: CurrentUser,
) -> Result<Response, WebError> {
    let repository = visible(&state, &owner, &repo, Some(&user.0)).await?;
    owner_only(&repository, &user.0)?;

    let slug = repository.slug();
    // The directory goes first: if removing it fails the row survives and the request can be tried
    // again, whereas deleting the row first would strand a directory nobody can reach or clean up.
    if state.git.exists(&slug) {
        state.git.remove(&slug)?;
    }
    repositories::delete(&state.db, repository.id.get()).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

/// `GET /api/v1/repos/{owner}/{repo}/branches`
pub async fn branches(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    viewer: MaybeUser,
) -> Result<Json<BranchList>, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let git = state.git.open(&repository.slug())?;
    let branches = read::branches(&git, &repository.default_branch)?;

    Ok(Json(BranchList { items: branches }))
}

/// The branch list, wrapped so it can grow fields without breaking a client.
#[derive(Debug, Serialize)]
pub struct BranchList {
    /// Branches, default branch first.
    pub items: Vec<Branch>,
}

/// `GET /api/v1/repos/{owner}/{repo}/tags`
pub async fn tags(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    viewer: MaybeUser,
) -> Result<Json<TagList>, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let git = state.git.open(&repository.slug())?;

    Ok(Json(TagList {
        items: read::tags(&git)?,
    }))
}

/// The tag list.
#[derive(Debug, Serialize)]
pub struct TagList {
    /// Tags, in git's own order.
    pub items: Vec<Tag>,
}

/// `GET /api/v1/repos/{owner}/{repo}/stats`
pub async fn stats(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    viewer: MaybeUser,
) -> Result<Json<GitStats>, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let stats = state
        .git
        .stats(&repository.slug(), &repository.default_branch)?;

    Ok(Json(stats))
}

/// Query for a commit list.
#[derive(Debug, Deserialize)]
pub struct CommitsQuery {
    /// Revision to walk, defaulting to the repository's default branch.
    #[serde(rename = "ref")]
    pub reference: Option<String>,
    /// Page number.
    pub page: Option<u32>,
    /// Items per page.
    pub per_page: Option<u32>,
}

/// A commit as the API returns it, with the abbreviation the UI shows alongside the full id.
#[derive(Debug, Serialize)]
pub struct CommitDto {
    #[serde(flatten)]
    commit: CommitSummary,
    short_sha: String,
}

impl CommitDto {
    fn new(commit: CommitSummary) -> Self {
        Self {
            short_sha: commit.short_sha(),
            commit,
        }
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/commits`
pub async fn commits(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<CommitsQuery>,
    uri: Uri,
    viewer: MaybeUser,
) -> Result<Response, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let page = PageQuery {
        page: query.page,
        per_page: query.per_page,
    }
    .page();
    let revision = query
        .reference
        .unwrap_or_else(|| repository.default_branch.clone());

    let git = state.git.open(&repository.slug())?;
    let skip = usize::try_from(page.offset()).unwrap_or(usize::MAX);
    let limit = usize::try_from(page.limit()).unwrap_or(usize::MAX);
    let history = read::log(&git, &revision, skip, limit)?;
    let counted = read::commit_count(&git, &revision, COMMIT_COUNT_CAP)?;

    let items: Vec<CommitDto> = history.into_iter().map(CommitDto::new).collect();
    let response = match counted {
        // The count is capped well below `i64::MAX`, so the conversion cannot lose anything.
        Some(total) => page_response(&state, &uri, items, page, total as i64),
        None => {
            // The history is longer than the cap, so no honest page count exists. The envelope says
            // `null` and the Link header offers a next page only, which is the whole truth.
            let body = scabby_core::pagination::PageResult::uncounted(items, page);
            let has_more = body.is_full();
            let mut response = Json(body).into_response();
            if let Some(link) =
                page.link_header_without_total(&crate::api::page_url(&state, &uri), has_more)
                && let Ok(value) = axum::http::HeaderValue::from_str(&link)
            {
                response.headers_mut().insert(header::LINK, value);
            }
            response
        }
    };

    Ok(response)
}

/// `GET /api/v1/repos/{owner}/{repo}/commits/{sha}`
pub async fn commit(
    State(state): State<AppState>,
    Path((owner, repo, sha)): Path<(String, String, String)>,
    viewer: MaybeUser,
) -> Result<Json<CommitDetailDto>, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let git = state.git.open(&repository.slug())?;

    let (summary, body) = read::commit_detail(&git, &sha)?;
    let parent = summary.parents.first().map(|parent| parent.to_string());
    // The file list comes from the git binary rather than from a reimplementation of rename
    // detection, which is the one place where shelling out is clearly the smaller program.
    let files = diff::file_changes(
        &state.git.path(&repository.slug()),
        &summary.sha.to_string(),
        parent.as_deref(),
    )?;

    let detail = read::detail(summary, body, files);
    let short_sha = detail.summary.short_sha();
    Ok(Json(CommitDetailDto { short_sha, detail }))
}

/// A commit with its change set. `detail` already carries totals for `files`.
#[derive(Debug, Serialize)]
pub struct CommitDetailDto {
    #[serde(flatten)]
    detail: CommitDetail,
    /// Abbreviation of the commit id, the form the UI shows.
    short_sha: String,
}

/// Query for a directory or file read.
#[derive(Debug, Deserialize)]
pub struct RevisionQuery {
    /// Revision to read at, defaulting to the repository's default branch.
    #[serde(rename = "ref")]
    pub reference: Option<String>,
}

/// `GET /api/v1/repos/{owner}/{repo}/contents`
pub async fn contents_root(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<RevisionQuery>,
    viewer: MaybeUser,
) -> Result<Response, WebError> {
    contents_at(state, owner, repo, String::new(), query, viewer).await
}

/// `GET /api/v1/repos/{owner}/{repo}/contents/{path}`
pub async fn contents(
    State(state): State<AppState>,
    Path((owner, repo, path)): Path<(String, String, String)>,
    Query(query): Query<RevisionQuery>,
    viewer: MaybeUser,
) -> Result<Response, WebError> {
    contents_at(state, owner, repo, path, query, viewer).await
}

/// A directory at a revision.
#[derive(Debug, Serialize)]
pub struct DirectoryResponse {
    #[serde(rename = "type")]
    kind: &'static str,
    path: String,
    entries: Vec<TreeEntry>,
}

/// A file at a revision.
#[derive(Debug, Serialize)]
pub struct FileResponse {
    #[serde(rename = "type")]
    kind: &'static str,
    #[serde(flatten)]
    blob: Blob,
}

/// Read a path at a revision, answering with a listing or a file.
async fn contents_at(
    state: AppState,
    owner: String,
    repo: String,
    path: String,
    query: RevisionQuery,
    viewer: MaybeUser,
) -> Result<Response, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let revision = query
        .reference
        .unwrap_or_else(|| repository.default_branch.clone());
    let git = state.git.open(&repository.slug())?;

    // A path is asked about without saying whether it is a directory or a file, so the listing is
    // tried first and a missing path is retried as a file rather than reported as not found.
    match read::tree(&git, &revision, &path) {
        Ok(listing) => Ok(Json(DirectoryResponse {
            kind: "dir",
            path: listing.path,
            entries: listing.entries,
        })
        .into_response()),
        Err(GitError::MissingPath { .. }) => {
            let blob = read::blob(&git, &revision, &path)?;
            Ok(Json(FileResponse { kind: "file", blob }).into_response())
        }
        Err(error) => Err(error.into()),
    }
}

/// `GET /api/v1/repos/{owner}/{repo}/raw/{ref}/{path}`
pub async fn raw(
    State(state): State<AppState>,
    Path((owner, repo, revision, path)): Path<(String, String, String, String)>,
    viewer: MaybeUser,
) -> Result<Response, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let git = state.git.open(&repository.slug())?;
    let (_, data) = read::blob_bytes(&git, &revision, &path)?;

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime.as_ref().to_owned()),
            // Serving user content from the instance's own origin is the one place a stored file
            // could be made to run as a page, so it is sent as an attachment of unknown type and
            // never sniffed.
            (
                header::CONTENT_DISPOSITION,
                format!("inline; filename=\"{}\"", file_name(&path)),
            ),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_owned()),
        ],
        Body::from(data),
    )
        .into_response())
}

/// Last path component, with anything that could break out of the header removed.
fn file_name(path: &str) -> String {
    path.rsplit('/')
        .next()
        .unwrap_or("file")
        .chars()
        .filter(|character| !matches!(character, '"' | '\\' | '\n' | '\r'))
        .collect()
}

/// `GET /api/v1/repos/{owner}/{repo}/readme`
pub async fn readme(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<RevisionQuery>,
    viewer: MaybeUser,
) -> Result<Json<ReadmeBody>, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let revision = query
        .reference
        .unwrap_or_else(|| repository.default_branch.clone());
    let git = state.git.open(&repository.slug())?;

    match read::readme(&git, &revision)? {
        Some((path, text)) => Ok(Json(ReadmeBody { path, text })),
        None => Err(WebError::Domain(Error::not_found(
            "readme",
            format!("{} has no readme at {revision}", repository.full_name()),
        ))),
    }
}

/// A repository's readme, as text for the client to render.
#[derive(Debug, Serialize)]
pub struct ReadmeBody {
    /// Path the readme was found at.
    pub path: String,
    /// Its contents.
    pub text: String,
}

/// Load a repository the caller is allowed to see.
pub async fn visible(
    state: &AppState,
    owner: &str,
    repo: &str,
    viewer: Option<&scabby_core::user::User>,
) -> Result<Repository, WebError> {
    let slug = Slug::parse(&format!("{owner}/{repo}"))?;
    let repository = repositories::by_slug(&state.db, &slug).await?;
    let is_owner = viewer.is_some_and(|user| user.id == repository.owner.id);

    if repository.visibility.is_private() && !is_owner {
        // A private repository answers exactly like one that does not exist, because telling a
        // stranger that a name is taken is itself a disclosure.
        return Err(WebError::Domain(Error::not_found(
            "repository",
            slug.full_name(),
        )));
    }

    Ok(repository)
}

/// Refuse a write from anyone but the owner.
pub fn owner_only(repository: &Repository, user: &scabby_core::user::User) -> Result<(), WebError> {
    if user.id != repository.owner.id {
        return Err(WebError::forbidden("only the repository owner may do that"));
    }
    Ok(())
}

/// A repository with its on-disk measurements attached, when the directory is there.
fn measure(state: &AppState, repository: Repository) -> RepositoryDto {
    let dto = RepositoryDto::new(state, repository.clone());
    if !state.git.exists(&repository.slug()) {
        return dto;
    }

    match state
        .git
        .stats(&repository.slug(), &repository.default_branch)
    {
        Ok(stats) => dto.with_stats(stats),
        Err(error) => {
            tracing::warn!(repository = %repository.full_name(), %error, "could not measure a repository");
            dto
        }
    }
}

/// Reject a default branch that the repository does not have.
fn ensure_branch_exists(
    state: &AppState,
    repository: &Repository,
    branch: &str,
) -> Result<(), WebError> {
    if !state.git.exists(&repository.slug()) {
        return Ok(());
    }

    let git = state.git.open(&repository.slug())?;
    let branches = read::branches(&git, &repository.default_branch)?;
    if branches.iter().any(|existing| existing.name == branch) {
        return Ok(());
    }

    Err(WebError::validation(
        "default_branch",
        "is not a branch in this repository",
    ))
}
