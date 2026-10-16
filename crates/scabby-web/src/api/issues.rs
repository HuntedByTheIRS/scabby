//! Issues and their comments.

use axum::Json;
use axum::Router;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::http::Uri;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use scabby_core::event::EventKind;
use scabby_core::issue::{Issue, IssueComment, IssueFilter, IssueState};
use scabby_db::{events, issues};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::repos::{owner_only, visible};
use crate::api::{PageQuery, field, page_response};
use crate::auth::{CurrentUser, MaybeUser};
use crate::{AppState, WebError};

/// Issue routes, all nested under a repository.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/repos/{owner}/{repo}/issues", get(list).post(create))
        .route(
            "/repos/{owner}/{repo}/issues/{number}",
            get(one).patch(update),
        )
        .route(
            "/repos/{owner}/{repo}/issues/{number}/comments",
            get(comments).post(comment),
        )
}

/// An issue as the API returns it: the stored fields plus the URL a client would otherwise build.
#[derive(Debug, Serialize)]
pub struct IssueDto {
    #[serde(flatten)]
    issue: Issue,
    html_url: String,
}

impl IssueDto {
    fn new(state: &AppState, repository: &scabby_core::repo::Repository, issue: Issue) -> Self {
        Self {
            html_url: state.html_url(&format!(
                "{}/issues/{}",
                repository.full_name(),
                issue.number
            )),
            issue,
        }
    }
}

/// Query for the issue list.
#[derive(Debug, Deserialize)]
pub struct IssueListQuery {
    /// `open`, `closed`, or `all`. Defaults to `open`.
    pub state: Option<String>,
    /// Page number.
    pub page: Option<u32>,
    /// Items per page.
    pub per_page: Option<u32>,
}

/// `GET /api/v1/repos/{owner}/{repo}/issues`
pub async fn list(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    Query(query): Query<IssueListQuery>,
    uri: Uri,
    viewer: MaybeUser,
) -> Result<Response, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let page = PageQuery {
        page: query.page,
        per_page: query.per_page,
    }
    .page();
    let filter = match query.state.as_deref() {
        None => IssueFilter::Open,
        Some(raw) => IssueFilter::parse(raw)?,
    };

    let (found, total) = issues::list(&state.db, repository.id.get(), filter, page).await?;
    let items: Vec<IssueDto> = found
        .into_iter()
        .map(|issue| IssueDto::new(&state, &repository, issue))
        .collect();

    Ok(page_response(&state, &uri, items, page, total))
}

/// Body for opening an issue.
#[derive(Debug, Deserialize)]
pub struct CreateIssueRequest {
    title: String,
    #[serde(default)]
    body: Option<String>,
}

/// `POST /api/v1/repos/{owner}/{repo}/issues`
pub async fn create(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
    user: CurrentUser,
    Json(request): Json<CreateIssueRequest>,
) -> Result<Response, WebError> {
    let repository = visible(&state, &owner, &repo, Some(&user.0)).await?;
    let title = require_title(&request.title)?;
    let body = request
        .body
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty());

    let issue = issues::create(
        &state.db,
        repository.id.get(),
        user.0.id.get(),
        &title,
        body,
    )
    .await?;

    let _ = events::record(
        &state.db,
        Some(user.0.id.get()),
        Some(repository.id.get()),
        EventKind::IssueOpened,
    )
    .await;

    Ok((
        StatusCode::CREATED,
        Json(IssueDto::new(&state, &repository, issue)),
    )
        .into_response())
}

/// `GET /api/v1/repos/{owner}/{repo}/issues/{number}`
pub async fn one(
    State(state): State<AppState>,
    Path((owner, repo, number)): Path<(String, String, i32)>,
    viewer: MaybeUser,
) -> Result<Json<IssueDto>, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let issue = issues::by_number(&state.db, repository.id.get(), number).await?;

    Ok(Json(IssueDto::new(&state, &repository, issue)))
}

/// `PATCH /api/v1/repos/{owner}/{repo}/issues/{number}`
pub async fn update(
    State(state): State<AppState>,
    Path((owner, repo, number)): Path<(String, String, i32)>,
    user: CurrentUser,
    Json(body): Json<Value>,
) -> Result<Json<IssueDto>, WebError> {
    let repository = visible(&state, &owner, &repo, Some(&user.0)).await?;
    let issue = issues::by_number(&state.db, repository.id.get(), number).await?;

    // Editing an issue is the author's or the repository owner's business; anyone else can comment.
    if issue.author.id != user.0.id {
        owner_only(&repository, &user.0)?;
    }

    let title: Option<String> = field(&body, "title")?;
    let text: Option<String> = field(&body, "body")?;
    // Named for what it is: the `state` name here belongs to the router's shared state.
    let new_state: Option<IssueState> = field::<String>(&body, "state")?
        .map(|raw| IssueState::parse(&raw))
        .transpose()?;

    let patch = issues::IssuePatch {
        title: title.as_deref().map(require_title).transpose()?,
        body: text,
        state: new_state,
    };

    let updated = issues::update(&state.db, repository.id.get(), number, &patch).await?;
    if matches!(updated.state, IssueState::Closed) && !matches!(issue.state, IssueState::Closed) {
        let _ = events::record(
            &state.db,
            Some(user.0.id.get()),
            Some(repository.id.get()),
            EventKind::IssueClosed,
        )
        .await;
    }

    Ok(Json(IssueDto::new(&state, &repository, updated)))
}

/// `GET /api/v1/repos/{owner}/{repo}/issues/{number}/comments`
pub async fn comments(
    State(state): State<AppState>,
    Path((owner, repo, number)): Path<(String, String, i32)>,
    Query(query): Query<PageQuery>,
    uri: Uri,
    viewer: MaybeUser,
) -> Result<Response, WebError> {
    let repository = visible(&state, &owner, &repo, viewer.0.as_ref()).await?;
    let page = query.page();
    let (found, total) = issues::comments(&state.db, repository.id.get(), number, page).await?;

    Ok(page_response(&state, &uri, found, page, total))
}

/// Body for adding a comment.
#[derive(Debug, Deserialize)]
pub struct CommentRequest {
    body: String,
}

/// `POST /api/v1/repos/{owner}/{repo}/issues/{number}/comments`
pub async fn comment(
    State(state): State<AppState>,
    Path((owner, repo, number)): Path<(String, String, i32)>,
    user: CurrentUser,
    Json(request): Json<CommentRequest>,
) -> Result<Response, WebError> {
    let repository = visible(&state, &owner, &repo, Some(&user.0)).await?;

    let body = request.body.trim();
    if body.is_empty() {
        return Err(WebError::validation("body", "must not be empty"));
    }

    let comment: IssueComment = issues::add_comment(
        &state.db,
        repository.id.get(),
        number,
        user.0.id.get(),
        body,
    )
    .await?;

    let _ = events::record(
        &state.db,
        Some(user.0.id.get()),
        Some(repository.id.get()),
        EventKind::IssueCommented,
    )
    .await;

    Ok((StatusCode::CREATED, Json(comment)).into_response())
}

/// Trim a title and refuse an empty one, naming the field the client sent.
fn require_title(title: &str) -> Result<String, WebError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(WebError::validation("title", "must not be empty"));
    }
    Ok(title.to_owned())
}
