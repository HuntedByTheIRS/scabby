//! Issues and comments, numbered within their repository.

use scabby_core::issue::{Issue, IssueComment, IssueFilter, IssueState};
use scabby_core::pagination::Page;
use time::OffsetDateTime;

use crate::rows::OwnerColumns;
use crate::sql;
use crate::{Db, DbError, Result};

#[derive(Debug, Clone, sqlx::FromRow)]
struct IssueRow {
    id: i64,
    number: i32,
    title: String,
    body: Option<String>,
    state: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    closed_at: Option<OffsetDateTime>,
    #[sqlx(flatten)]
    author: OwnerColumns,
    comment_count: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct CommentRow {
    id: i64,
    issue_number: i32,
    body: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    #[sqlx(flatten)]
    author: OwnerColumns,
}

fn select_issues() -> String {
    format!(
        "select i.id, i.number, i.title, i.body, i.state, i.created_at, i.updated_at, i.closed_at,{author},
                (select count(*) from issue_comments c where c.issue_id = i.id) as comment_count
         from issues i join users u on u.id = i.author_id",
        author = OwnerColumns::SELECT
    )
}

fn select_comments() -> String {
    format!(
        "select c.id, i.number as issue_number, c.body, c.created_at, c.updated_at,{author}
         from issue_comments c
         join issues i on i.id = c.issue_id
         join users u on u.id = c.author_id",
        author = OwnerColumns::SELECT
    )
}

impl IssueRow {
    fn into_issue(self) -> Result<Issue> {
        Ok(Issue {
            id: self.id.into(),
            number: self.number,
            title: self.title,
            body: self.body,
            state: IssueState::parse(&self.state)?,
            author: self.author.into_user_ref()?,
            comment_count: self.comment_count,
            created_at: self.created_at.into(),
            updated_at: self.updated_at.into(),
            closed_at: self.closed_at.map(Into::into),
        })
    }
}

impl CommentRow {
    fn into_comment(self) -> Result<IssueComment> {
        Ok(IssueComment {
            id: self.id.into(),
            issue_number: self.issue_number,
            author: self.author.into_user_ref()?,
            body: self.body,
            created_at: self.created_at.into(),
            updated_at: self.updated_at.into(),
        })
    }
}

/// Open an issue, taking the next number for its repository.
///
/// The repository row is locked first, so two concurrent creates cannot hand out the same number:
/// the unique index on `(repo_id, number)` would otherwise turn a race into a failed request.
pub async fn create(
    db: &Db,
    repo_id: i64,
    author_id: i64,
    title: &str,
    body: Option<&str>,
) -> Result<Issue> {
    let mut tx = db.pool().begin().await?;

    let locked: Option<i64> =
        sqlx::query_scalar("select id from repositories where id = $1 for update")
            .bind(repo_id)
            .fetch_optional(&mut *tx)
            .await?;
    if locked.is_none() {
        return Err(DbError::not_found("repository", repo_id));
    }

    let number: i32 =
        sqlx::query_scalar("select coalesce(max(number), 0) + 1 from issues where repo_id = $1")
            .bind(repo_id)
            .fetch_one(&mut *tx)
            .await?;

    let id: i64 = sqlx::query_scalar(
        "insert into issues (repo_id, number, title, body, author_id)
         values ($1, $2, $3, $4, $5)
         returning id",
    )
    .bind(repo_id)
    .bind(number)
    .bind(title)
    .bind(body)
    .bind(author_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(|error| DbError::from(error).or_conflict("issue", title))?;

    tx.commit().await?;

    let statement = sql::fragment(format!("{} where i.id = $1", select_issues()));
    let row: Option<IssueRow> = sqlx::query_as(statement)
        .bind(id)
        .fetch_optional(db.pool())
        .await?;

    match row {
        Some(row) => row.into_issue(),
        None => Err(DbError::not_found("issue", id)),
    }
}

/// Look up one issue by its per-repository number.
pub async fn by_number(db: &Db, repo_id: i64, number: i32) -> Result<Issue> {
    let statement = sql::fragment(format!(
        "{} where i.repo_id = $1 and i.number = $2",
        select_issues()
    ));

    let row: Option<IssueRow> = sqlx::query_as(statement)
        .bind(repo_id)
        .bind(number)
        .fetch_optional(db.pool())
        .await?;

    match row {
        Some(row) => row.into_issue(),
        None => Err(DbError::not_found("issue", number)),
    }
}

/// List a repository's issues, newest number first.
pub async fn list(
    db: &Db,
    repo_id: i64,
    filter: IssueFilter,
    page: Page,
) -> Result<(Vec<Issue>, i64)> {
    let state = filter.as_state().map(IssueState::as_str);

    let statement = sql::fragment(format!(
        "{} where i.repo_id = $1 and ($2::text is null or i.state = $2)
         order by i.number desc
         limit $3 offset $4",
        select_issues()
    ));

    let rows: Vec<IssueRow> = sqlx::query_as(statement)
        .bind(repo_id)
        .bind(state)
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(db.pool())
        .await?;

    let total: i64 = sqlx::query_scalar(
        "select count(*) from issues where repo_id = $1 and ($2::text is null or state = $2)",
    )
    .bind(repo_id)
    .bind(state)
    .fetch_one(db.pool())
    .await?;

    let issues = rows
        .into_iter()
        .map(IssueRow::into_issue)
        .collect::<Result<Vec<_>>>()?;
    Ok((issues, total))
}

/// The fields an issue form may change.
#[derive(Debug, Clone, Default)]
pub struct IssuePatch {
    /// New title.
    pub title: Option<String>,
    /// New body.
    pub body: Option<String>,
    /// New state, which also stamps or clears `closed_at`.
    pub state: Option<IssueState>,
}

/// Apply a patch to an issue.
pub async fn update(db: &Db, repo_id: i64, number: i32, patch: &IssuePatch) -> Result<Issue> {
    let state = patch.state.map(IssueState::as_str);

    let changed = sqlx::query(
        "update issues
         set title = coalesce($3, title),
             body = coalesce($4, body),
             state = coalesce($5, state),
             closed_at = case when $5 = 'closed' then coalesce(closed_at, now())
                              when $5 = 'open' then null
                              else closed_at end,
             updated_at = now()
         where repo_id = $1 and number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .bind(patch.title.as_deref())
    .bind(patch.body.as_deref())
    .bind(state)
    .execute(db.pool())
    .await?
    .rows_affected();

    if changed == 0 {
        return Err(DbError::not_found("issue", number));
    }
    by_number(db, repo_id, number).await
}

/// Comments on an issue, oldest first.
pub async fn comments(
    db: &Db,
    repo_id: i64,
    number: i32,
    page: Page,
) -> Result<(Vec<IssueComment>, i64)> {
    let statement = sql::fragment(format!(
        "{} where i.repo_id = $1 and i.number = $2
         order by c.created_at asc, c.id asc
         limit $3 offset $4",
        select_comments()
    ));

    let rows: Vec<CommentRow> = sqlx::query_as(statement)
        .bind(repo_id)
        .bind(number)
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(db.pool())
        .await?;

    let total: i64 = sqlx::query_scalar(
        "select count(*) from issue_comments c join issues i on i.id = c.issue_id
         where i.repo_id = $1 and i.number = $2",
    )
    .bind(repo_id)
    .bind(number)
    .fetch_one(db.pool())
    .await?;

    let comments = rows
        .into_iter()
        .map(CommentRow::into_comment)
        .collect::<Result<Vec<_>>>()?;
    Ok((comments, total))
}

/// Add a comment, touching the issue's `updated_at` in the same transaction.
pub async fn add_comment(
    db: &Db,
    repo_id: i64,
    number: i32,
    author_id: i64,
    body: &str,
) -> Result<IssueComment> {
    let mut tx = db.pool().begin().await?;

    // The insert selects the issue in the same statement, so a comment cannot land on a number that
    // does not exist, and no separate lookup can go stale between the read and the write.
    let id: Option<i64> = sqlx::query_scalar(
        "insert into issue_comments (issue_id, author_id, body)
         select i.id, $3, $4 from issues i where i.repo_id = $1 and i.number = $2
         returning id",
    )
    .bind(repo_id)
    .bind(number)
    .bind(author_id)
    .bind(body)
    .fetch_optional(&mut *tx)
    .await
    .map_err(DbError::from)?;

    let Some(id) = id else {
        return Err(DbError::not_found("issue", number));
    };

    sqlx::query("update issues set updated_at = now() where repo_id = $1 and number = $2")
        .bind(repo_id)
        .bind(number)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    let statement = sql::fragment(format!("{} where c.id = $1", select_comments()));
    let row: Option<CommentRow> = sqlx::query_as(statement)
        .bind(id)
        .fetch_optional(db.pool())
        .await?;

    match row {
        Some(row) => row.into_comment(),
        None => Err(DbError::not_found("issue comment", id)),
    }
}
