//! The activity feed.

use scabby_core::event::{Event, EventKind};
use scabby_core::names::{Name, Slug};
use scabby_core::pagination::Page;
use scabby_core::user::UserRef;
use time::OffsetDateTime;

use crate::sql;
use crate::{Db, Result};

#[derive(Debug, Clone, sqlx::FromRow)]
struct EventRow {
    id: i64,
    kind: String,
    created_at: OffsetDateTime,
    actor_id: Option<i64>,
    actor_username: Option<String>,
    actor_display_name: Option<String>,
    actor_avatar_url: Option<String>,
    repo_owner: Option<String>,
    repo_name: Option<String>,
}

impl EventRow {
    fn into_event(self) -> Result<Event> {
        let actor = match (self.actor_id, self.actor_username) {
            (Some(id), Some(username)) => Some(UserRef {
                id: id.into(),
                username: Name::parse(&username)?,
                display_name: self.actor_display_name,
                avatar_url: self.actor_avatar_url,
            }),
            _ => None,
        };

        let repository = match (self.repo_owner, self.repo_name) {
            (Some(owner), Some(name)) => Some(Slug::new(Name::parse(&owner)?, Name::parse(&name)?)),
            _ => None,
        };

        Ok(Event {
            id: self.id.into(),
            kind: EventKind::parse(&self.kind)?,
            actor,
            repository,
            created_at: self.created_at.into(),
        })
    }
}

/// The feed joins outward twice: once for whoever acted, once for the repository's owner, because a
/// repository row on its own does not carry the `owner/name` the feed renders.
const SELECT: &str = r#"
    select e.id, e.kind, e.created_at,
           a.id as actor_id, a.username as actor_username,
           a.display_name as actor_display_name, a.avatar_url as actor_avatar_url,
           o.username as repo_owner, r.name as repo_name
    from events e
    left join users a on a.id = e.actor_id
    left join repositories r on r.id = e.repo_id
    left join users o on o.id = r.owner_id"#;

/// Append an entry to the feed.
///
/// Events are recorded after the thing they describe succeeds, and a missing actor or repository is
/// accepted: a push by a deleted user still deserves a line rather than a hole in the feed.
pub async fn record(
    db: &Db,
    actor_id: Option<i64>,
    repo_id: Option<i64>,
    kind: EventKind,
) -> Result<()> {
    sqlx::query("insert into events (actor_id, repo_id, kind) values ($1, $2, $3)")
        .bind(actor_id)
        .bind(repo_id)
        .bind(kind.as_str())
        .execute(db.pool())
        .await?;
    Ok(())
}

/// The feed, newest first.
pub async fn recent(db: &Db, page: Page) -> Result<(Vec<Event>, i64)> {
    let statement = sql::fragment(format!(
        "{SELECT} order by e.created_at desc, e.id desc limit $1 offset $2"
    ));

    let rows: Vec<EventRow> = sqlx::query_as(statement)
        .bind(page.limit())
        .bind(page.offset())
        .fetch_all(db.pool())
        .await?;

    let total: i64 = sqlx::query_scalar("select count(*) from events")
        .fetch_one(db.pool())
        .await?;

    let events = rows
        .into_iter()
        .map(EventRow::into_event)
        .collect::<Result<Vec<_>>>()?;
    Ok((events, total))
}

/// Drop entries older than the given date, so the feed cannot grow without bound.
pub async fn prune(db: &Db, older_than: OffsetDateTime) -> Result<u64> {
    let deleted = sqlx::query("delete from events where created_at < $1")
        .bind(older_than)
        .execute(db.pool())
        .await?
        .rows_affected();
    Ok(deleted)
}
