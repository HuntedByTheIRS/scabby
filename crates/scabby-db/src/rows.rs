//! Row shapes shared by more than one module.

use scabby_core::names::Name;
use scabby_core::user::UserRef;

/// The columns every query selects for the account that owns or authored something.
///
/// Kept in one place so a joined row can become a [`UserRef`] the same way everywhere.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OwnerColumns {
    /// Account id.
    pub owner_id: i64,
    /// Handle.
    pub owner_username: String,
    /// Display name, when set.
    pub owner_display_name: Option<String>,
    /// External avatar, when set.
    pub owner_avatar_url: Option<String>,
}

impl OwnerColumns {
    /// The `select` fragment that fills these columns from a join on `users`.
    pub const SELECT: &'static str = r#"
        u.id as owner_id, u.username as owner_username,
        u.display_name as owner_display_name, u.avatar_url as owner_avatar_url"#;

    /// Build the reference, validating the stored handle.
    pub fn into_user_ref(self) -> Result<UserRef, scabby_core::Error> {
        Ok(UserRef {
            id: self.owner_id.into(),
            username: Name::parse(&self.owner_username)?,
            display_name: self.owner_display_name,
            avatar_url: self.owner_avatar_url,
        })
    }
}
