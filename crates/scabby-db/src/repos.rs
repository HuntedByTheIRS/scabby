//! Repositories.

use scabby_core::names::{Name, Slug};
use scabby_core::pagination::Page;
use scabby_core::repo::{Repository, Visibility};
use time::OffsetDateTime;

use crate::rows::OwnerColumns;
use crate::sql;
use crate::{Db, DbError, Result};

#[derive(Debug, Clone, sqlx::FromRow)]
struct RepositoryRow {
    id: i64,
    name: String,
    description: Option<String>,
    visibility: String,
    default_branch: String,
    size_bytes: i64,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    #[sqlx(flatten)]
    owner: OwnerColumns,
    open_issue_count: i64,
}

/// The projection every read uses, so a repository is shaped in one place.
fn select() -> String {
    format!(
        "select r.id, r.name, r.description, r.visibility, r.default_branch, r.size_bytes,
                r.created_at, r.updated_at,{owner},
                (select count(*) from issues i where i.repo_id = r.id and i.state = 'open')
                    as open_issue_count
         from repositories r join users u on u.id = r.owner_id",
        owner = OwnerColumns::SELECT
    )
}

impl RepositoryRow {
    fn into_repository(self) -> Result<Repository> {
        Ok(Repository {
            id: self.id.into(),
            owner: self.owner.into_user_ref()?,
            name: Name::parse(&self.name)?,
            description: self.description,
            visibility: Visibility::parse(&self.visibility)?,
            default_branch: self.default_branch,
            size_bytes: self.size_bytes,
            open_issue_count: self.open_issue_count,
            created_at: self.created_at.into(),
            updated_at: self.updated_at.into(),
        })
    }
}

/// Create the database row for a repository. The bare repository on disk is the git layer's job.
pub async fn create(
    db: &Db,
    owner_id: i64,
    name: &Name,
    description: Option<&str>,
    visibility: Visibility,
    default_branch: &str,
) -> Result<Repository> {
    let id: i64 = sqlx::query_scalar(
        "insert into repositories (owner_id, name, description, visibility, default_branch)
         values ($1, $2, $3, $4, $5)
         returning id",
    )
    .bind(owner_id)
    .bind(name.as_str())
    .bind(description)
    .bind(visibility.as_str())
    .bind(default_branch)
    .fetch_one(db.pool())
    .await
    .map_err(|error| DbError::from(error).or_conflict("repository", name.as_str()))?;

    by_id(db, id).await
}

/// Look up a repository by `owner/name`, ignoring case on both halves.
pub async fn by_slug(db: &Db, slug: &Slug) -> Result<Repository> {
    let statement = sql::fragment(format!(
        "{} where u.username_lower = lower($1) and r.name_lower = lower($2)",
        select()
    ));

    let row: Option<RepositoryRow> = sqlx::query_as(statement)
        .bind(slug.owner.as_str())
        .bind(slug.name.as_str())
        .fetch_optional(db.pool())
        .await?;

    match row {
        Some(row) => row.into_repository(),
        None => Err(DbError::not_found("repository", slug.full_name())),
    }
}

/// Look up a repository by id.
pub async fn by_id(db: &Db, id: i64) -> Result<Repository> {
    let statement = sql::fragment(format!("{} where r.id = $1", select()));

    let row: Option<RepositoryRow> = sqlx::query_as(statement)
        .bind(id)
        .fetch_optional(db.pool())
        .await?;

    match row {
        Some(row) => row.into_repository(),
        None => Err(DbError::not_found("repository", id)),
    }
}

/// Repositories the viewer may see: everything public, plus their own private ones.
pub async fn list_visible(
    db: &Db,
    viewer: Option<i64>,
    page: Page,
) -> Result<(Vec<Repository>, i64)> {
    let statement = sql::fragment(format!(
        "{} where r.visibility = 'public' or r.owner_id = $1
         order by r.updated_at desc, r.id desc
         limit $2 offset $3",
        select()
    ));

    let rows: Vec<RepositoryRow> = sqlx::query_as(statement)
        .bind(viewer)
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(db.pool())
        .await?;

    let total: i64 = sqlx::query_scalar(
        "select count(*) from repositories r where r.visibility = 'public' or r.owner_id = $1",
    )
    .bind(viewer)
    .fetch_one(db.pool())
    .await?;

    let repositories = rows
        .into_iter()
        .map(RepositoryRow::into_repository)
        .collect::<Result<Vec<_>>>()?;
    Ok((repositories, total))
}

/// Repositories owned by one account, as seen by `viewer`.
pub async fn list_owned(
    db: &Db,
    owner_id: i64,
    viewer: Option<i64>,
    page: Page,
) -> Result<(Vec<Repository>, i64)> {
    let statement = sql::fragment(format!(
        "{} where r.owner_id = $1 and (r.visibility = 'public' or r.owner_id = $2)
         order by r.updated_at desc, r.id desc
         limit $3 offset $4",
        select()
    ));

    let rows: Vec<RepositoryRow> = sqlx::query_as(statement)
        .bind(owner_id)
        .bind(viewer)
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(db.pool())
        .await?;

    let total: i64 = sqlx::query_scalar(
        "select count(*) from repositories
         where owner_id = $1 and (visibility = 'public' or owner_id = $2)",
    )
    .bind(owner_id)
    .bind(viewer)
    .fetch_one(db.pool())
    .await?;

    let repositories = rows
        .into_iter()
        .map(RepositoryRow::into_repository)
        .collect::<Result<Vec<_>>>()?;
    Ok((repositories, total))
}

/// The fields a settings form may change.
#[derive(Debug, Clone, Default)]
pub struct RepositoryPatch {
    /// New description. An empty string clears it; `None` leaves it alone.
    pub description: Option<String>,
    /// New visibility.
    pub visibility: Option<Visibility>,
    /// New default branch.
    pub default_branch: Option<String>,
}

/// Apply a patch. Every field is independent, so a form can save one without touching the rest.
pub async fn update(db: &Db, id: i64, patch: &RepositoryPatch) -> Result<Repository> {
    let changed = sqlx::query(
        "update repositories
         set description = case when $2::text is null then description else nullif($2, '') end,
             visibility = coalesce($3, visibility),
             default_branch = coalesce($4, default_branch),
             updated_at = now()
         where id = $1",
    )
    .bind(id)
    .bind(patch.description.as_deref())
    .bind(patch.visibility.map(Visibility::as_str))
    .bind(patch.default_branch.as_deref())
    .execute(db.pool())
    .await?
    .rows_affected();

    if changed == 0 {
        return Err(DbError::not_found("repository", id));
    }
    by_id(db, id).await
}

/// Record the on-disk size after a push, so the repository list does not have to walk the disk.
pub async fn set_size(db: &Db, id: i64, size_bytes: u64) -> Result<()> {
    let size = i64::try_from(size_bytes).unwrap_or(i64::MAX);
    sqlx::query("update repositories set size_bytes = $2, updated_at = now() where id = $1")
        .bind(id)
        .bind(size)
        .execute(db.pool())
        .await?;
    Ok(())
}

/// Remove a repository row. Issues, comments, and events go with it.
pub async fn delete(db: &Db, id: i64) -> Result<()> {
    let deleted = sqlx::query("delete from repositories where id = $1")
        .bind(id)
        .execute(db.pool())
        .await?
        .rows_affected();

    if deleted == 0 {
        return Err(DbError::not_found("repository", id));
    }
    Ok(())
}
