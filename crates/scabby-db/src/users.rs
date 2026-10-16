//! Accounts.

use scabby_core::names::Name;
use scabby_core::user::User;
use time::OffsetDateTime;

use crate::sql;
use crate::{Db, DbError, Result};

/// Every column the API exposes for an account, plus the password hash when asked for.
#[derive(Debug, Clone, sqlx::FromRow)]
struct UserRow {
    id: i64,
    username: String,
    display_name: Option<String>,
    email: String,
    bio: Option<String>,
    avatar_url: Option<String>,
    is_admin: bool,
    created_at: OffsetDateTime,
}

const COLUMNS: &str = r#"
    id, username, display_name, email, bio, avatar_url, is_admin, created_at"#;

impl UserRow {
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

/// An account together with the hash needed to check a password.
pub struct Credentials {
    /// The account.
    pub user: User,
    /// Argon2id hash, as stored.
    pub password_hash: String,
}

/// Create an account.
pub async fn create(
    db: &Db,
    username: &Name,
    email: &str,
    password_hash: &str,
    is_admin: bool,
) -> Result<User> {
    let statement = sql::fragment(format!(
        "insert into users (username, email, password_hash, is_admin)
         values ($1, $2, $3, $4)
         returning {COLUMNS}"
    ));

    let row: UserRow = sqlx::query_as(statement)
        .bind(username.as_str())
        .bind(email)
        .bind(password_hash)
        .bind(is_admin)
        .fetch_one(db.pool())
        .await
        .map_err(|error| DbError::from(error).or_conflict("user", username.as_str()))?;

    row.into_user(true)
}

/// Look up an account by handle, ignoring case.
pub async fn by_username(db: &Db, username: &str, reveal_email: bool) -> Result<User> {
    let statement = sql::fragment(format!(
        "select {COLUMNS} from users where username_lower = lower($1)"
    ));

    let row: Option<UserRow> = sqlx::query_as(statement)
        .bind(username)
        .fetch_optional(db.pool())
        .await?;

    match row {
        Some(row) => row.into_user(reveal_email),
        None => Err(DbError::not_found("user", username)),
    }
}

/// Look up an account by id.
pub async fn by_id(db: &Db, id: i64, reveal_email: bool) -> Result<User> {
    let statement = sql::fragment(format!("select {COLUMNS} from users where id = $1"));

    let row: Option<UserRow> = sqlx::query_as(statement)
        .bind(id)
        .fetch_optional(db.pool())
        .await?;

    match row {
        Some(row) => row.into_user(reveal_email),
        None => Err(DbError::not_found("user", id)),
    }
}

/// Look up an account by handle or email, for the login form.
pub async fn credentials(db: &Db, login: &str) -> Result<Credentials> {
    #[derive(sqlx::FromRow)]
    struct CredentialRow {
        #[sqlx(flatten)]
        user: UserRow,
        password_hash: String,
    }

    let statement = sql::fragment(format!(
        "select {COLUMNS}, password_hash from users
         where username_lower = lower($1) or email_lower = lower($1)"
    ));

    let row: Option<CredentialRow> = sqlx::query_as(statement)
        .bind(login)
        .fetch_optional(db.pool())
        .await?;

    match row {
        Some(row) => Ok(Credentials {
            user: row.user.into_user(true)?,
            password_hash: row.password_hash,
        }),
        None => Err(DbError::not_found("user", login)),
    }
}

/// Change the fields the profile form owns. A `None` leaves that field alone.
pub async fn update_profile(
    db: &Db,
    id: i64,
    display_name: Option<&str>,
    bio: Option<&str>,
) -> Result<User> {
    let statement = sql::fragment(format!(
        "update users
         set display_name = coalesce($2, display_name),
             bio = coalesce($3, bio),
             updated_at = now()
         where id = $1
         returning {COLUMNS}"
    ));

    let row: Option<UserRow> = sqlx::query_as(statement)
        .bind(id)
        .bind(display_name)
        .bind(bio)
        .fetch_optional(db.pool())
        .await?;

    match row {
        Some(row) => row.into_user(true),
        None => Err(DbError::not_found("user", id)),
    }
}

/// Whether any account exists, so the first registration can be made an administrator.
pub async fn count(db: &Db) -> Result<i64> {
    let total: i64 = sqlx::query_scalar("select count(*) from users")
        .fetch_one(db.pool())
        .await?;
    Ok(total)
}
