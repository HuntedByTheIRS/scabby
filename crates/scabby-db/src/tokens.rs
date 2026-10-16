//! Personal access tokens, browser sessions, and SSH public keys.

use scabby_core::auth::{ApiToken, SshKey};
use scabby_core::names::Name;
use scabby_core::user::User;
use time::OffsetDateTime;

use crate::sql;
use crate::{Db, DbError, Result};

#[derive(Debug, Clone, sqlx::FromRow)]
struct TokenRow {
    id: i64,
    name: String,
    prefix: String,
    scopes: Vec<String>,
    created_at: OffsetDateTime,
    last_used_at: Option<OffsetDateTime>,
    expires_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct KeyRow {
    id: i64,
    title: String,
    public_key: String,
    fingerprint: String,
    created_at: OffsetDateTime,
    last_used_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct SessionUserRow {
    id: i64,
    username: String,
    display_name: Option<String>,
    email: String,
    bio: Option<String>,
    avatar_url: Option<String>,
    is_admin: bool,
    created_at: OffsetDateTime,
}

const TOKEN_COLUMNS: &str = "id, name, prefix, scopes, created_at, last_used_at, expires_at";

const KEY_COLUMNS: &str = "id, title, public_key, fingerprint, created_at, last_used_at";

impl TokenRow {
    fn into_token(self) -> ApiToken {
        ApiToken {
            id: self.id.into(),
            name: self.name,
            prefix: self.prefix,
            scopes: self.scopes,
            created_at: self.created_at.into(),
            last_used_at: self.last_used_at.map(Into::into),
            expires_at: self.expires_at.map(Into::into),
        }
    }
}

impl KeyRow {
    fn into_key(self) -> SshKey {
        SshKey {
            id: self.id.into(),
            title: self.title,
            public_key: self.public_key,
            fingerprint: self.fingerprint,
            created_at: self.created_at.into(),
            last_used_at: self.last_used_at.map(Into::into),
        }
    }
}

impl SessionUserRow {
    fn into_user(self, reveal_email: bool) -> Result<User> {
        Ok(User {
            id: self.id.into(),
            username: Name::parse(&self.username)?,
            display_name: self.display_name,
            email: reveal_email.then_some(self.email),
            bio: self.bio,
            avatar_url: self.avatar_url,
            is_admin: self.is_admin,
            created_at: self.created_at.into(),
        })
    }
}

/// Store a token by hash. The secret never reaches this layer.
pub async fn create_token(
    db: &Db,
    user_id: i64,
    name: &str,
    prefix: &str,
    token_hash: &[u8],
    scopes: &[String],
    expires_at: Option<OffsetDateTime>,
) -> Result<ApiToken> {
    let statement = sql::fragment(format!(
        "insert into api_tokens (user_id, name, prefix, token_hash, scopes, expires_at)
         values ($1, $2, $3, $4, $5, $6)
         returning {TOKEN_COLUMNS}"
    ));

    let row: TokenRow = sqlx::query_as(statement)
        .bind(user_id)
        .bind(name)
        .bind(prefix)
        .bind(token_hash)
        .bind(scopes)
        .bind(expires_at)
        .fetch_one(db.pool())
        .await
        .map_err(DbError::from)?;

    Ok(row.into_token())
}

/// Every token a user has, newest first. The secrets are not retrievable by design.
pub async fn tokens_for_user(db: &Db, user_id: i64) -> Result<Vec<ApiToken>> {
    let statement = sql::fragment(format!(
        "select {TOKEN_COLUMNS} from api_tokens where user_id = $1 order by created_at desc"
    ));

    let rows: Vec<TokenRow> = sqlx::query_as(statement)
        .bind(user_id)
        .fetch_all(db.pool())
        .await?;

    Ok(rows.into_iter().map(TokenRow::into_token).collect())
}

/// Resolve a presented token to its owner, and record the use.
///
/// The lookup, the expiry check, and the `last_used_at` write are one statement, so a token cannot
/// be accepted after it expires because of a race with whoever reads it.
pub async fn token_user(db: &Db, token_hash: &[u8]) -> Result<Option<User>> {
    let row: Option<SessionUserRow> = sqlx::query_as(
        "update api_tokens t set last_used_at = now()
         from users u
         where t.user_id = u.id
           and t.token_hash = $1
           and (t.expires_at is null or t.expires_at > now())
         returning u.id, u.username, u.display_name, u.email, u.bio, u.avatar_url,
                   u.is_admin, u.created_at",
    )
    .bind(token_hash)
    .fetch_optional(db.pool())
    .await?;

    match row {
        Some(row) => Ok(Some(row.into_user(true)?)),
        None => Ok(None),
    }
}

/// Revoke one of a user's tokens.
pub async fn delete_token(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let deleted = sqlx::query("delete from api_tokens where id = $1 and user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(db.pool())
        .await?
        .rows_affected();

    if deleted == 0 {
        return Err(DbError::not_found("token", id));
    }
    Ok(())
}

/// Start a browser session.
pub async fn create_session(
    db: &Db,
    user_id: i64,
    token_hash: &[u8],
    user_agent: Option<&str>,
    expires_at: OffsetDateTime,
) -> Result<()> {
    sqlx::query(
        "insert into sessions (user_id, token_hash, user_agent, expires_at)
         values ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(user_agent)
    .bind(expires_at)
    .execute(db.pool())
    .await?;
    Ok(())
}

/// Resolve a session cookie to its owner, and record the visit.
pub async fn session_user(db: &Db, token_hash: &[u8]) -> Result<Option<User>> {
    let row: Option<SessionUserRow> = sqlx::query_as(
        "update sessions s set last_seen_at = now()
         from users u
         where s.user_id = u.id and s.token_hash = $1 and s.expires_at > now()
         returning u.id, u.username, u.display_name, u.email, u.bio, u.avatar_url,
                   u.is_admin, u.created_at",
    )
    .bind(token_hash)
    .fetch_optional(db.pool())
    .await?;

    match row {
        Some(row) => Ok(Some(row.into_user(true)?)),
        None => Ok(None),
    }
}

/// End a session.
pub async fn delete_session(db: &Db, token_hash: &[u8]) -> Result<()> {
    sqlx::query("delete from sessions where token_hash = $1")
        .bind(token_hash)
        .execute(db.pool())
        .await?;
    Ok(())
}

/// Remove expired sessions, so the table does not accumulate forever.
pub async fn prune_sessions(db: &Db) -> Result<u64> {
    let deleted = sqlx::query("delete from sessions where expires_at < now()")
        .execute(db.pool())
        .await?
        .rows_affected();
    Ok(deleted)
}

/// Register a public key. The fingerprint is unique across the instance.
pub async fn create_ssh_key(
    db: &Db,
    user_id: i64,
    title: &str,
    public_key: &str,
    fingerprint: &str,
) -> Result<SshKey> {
    let statement = sql::fragment(format!(
        "insert into ssh_keys (user_id, title, public_key, fingerprint)
         values ($1, $2, $3, $4)
         returning {KEY_COLUMNS}"
    ));

    let row: KeyRow = sqlx::query_as(statement)
        .bind(user_id)
        .bind(title)
        .bind(public_key)
        .bind(fingerprint)
        .fetch_one(db.pool())
        .await
        .map_err(|error| DbError::from(error).or_conflict("ssh key", fingerprint))?;

    Ok(row.into_key())
}

/// Every key a user has registered.
pub async fn ssh_keys_for_user(db: &Db, user_id: i64) -> Result<Vec<SshKey>> {
    let statement = sql::fragment(format!(
        "select {KEY_COLUMNS} from ssh_keys where user_id = $1 order by created_at desc"
    ));

    let rows: Vec<KeyRow> = sqlx::query_as(statement)
        .bind(user_id)
        .fetch_all(db.pool())
        .await?;

    Ok(rows.into_iter().map(KeyRow::into_key).collect())
}

/// Remove one of a user's keys.
pub async fn delete_ssh_key(db: &Db, user_id: i64, id: i64) -> Result<()> {
    let deleted = sqlx::query("delete from ssh_keys where id = $1 and user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(db.pool())
        .await?
        .rows_affected();

    if deleted == 0 {
        return Err(DbError::not_found("ssh key", id));
    }
    Ok(())
}
