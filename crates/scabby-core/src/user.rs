//! User accounts and the trimmed form other objects reference them by.

use serde::{Deserialize, Serialize};

use crate::ids::UserId;
use crate::names::Name;
use crate::timestamp::Timestamp;

/// A user account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Account identifier.
    pub id: UserId,
    /// Handle, unique across the instance and unique ignoring case.
    pub username: Name,
    /// Name to show in the UI, when the user set one.
    pub display_name: Option<String>,
    /// Email address. Present only on your own user object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Free-text profile blurb.
    pub bio: Option<String>,
    /// External avatar image, when the user set one.
    pub avatar_url: Option<String>,
    /// Whether the account can administer the instance.
    pub is_admin: bool,
    /// When the account was created.
    pub created_at: Timestamp,
}

impl User {
    /// The account as another object references it.
    pub fn as_ref_(&self) -> UserRef {
        UserRef {
            id: self.id,
            username: self.username.clone(),
            display_name: self.display_name.clone(),
            avatar_url: self.avatar_url.clone(),
        }
    }
}

/// A user as referenced from a repository, issue, or comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRef {
    /// Account identifier.
    pub id: UserId,
    /// Handle.
    pub username: Name,
    /// Name to show, when set.
    pub display_name: Option<String>,
    /// External avatar image, when set.
    pub avatar_url: Option<String>,
}
