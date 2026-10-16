//! Activity feed entries.

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::ids::EventId;
use crate::names::Slug;
use crate::timestamp::Timestamp;
use crate::user::UserRef;

/// What happened, in the order the feed sorts by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    /// An account was created.
    UserRegistered,
    /// A repository was created.
    RepositoryCreated,
    /// Commits arrived over an authenticated push.
    PushReceived,
    /// An issue was opened.
    IssueOpened,
    /// An issue was closed.
    IssueClosed,
    /// A comment was added to an issue.
    IssueCommented,
}

impl EventKind {
    /// The form stored in the database and serialized to the API.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserRegistered => "user_registered",
            Self::RepositoryCreated => "repository_created",
            Self::PushReceived => "push_received",
            Self::IssueOpened => "issue_opened",
            Self::IssueClosed => "issue_closed",
            Self::IssueCommented => "issue_commented",
        }
    }

    /// Parse the stored form.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "user_registered" => Ok(Self::UserRegistered),
            "repository_created" => Ok(Self::RepositoryCreated),
            "push_received" => Ok(Self::PushReceived),
            "issue_opened" => Ok(Self::IssueOpened),
            "issue_closed" => Ok(Self::IssueClosed),
            "issue_commented" => Ok(Self::IssueCommented),
            other => Err(Error::validation(
                "kind",
                format!("unknown event kind {other}"),
            )),
        }
    }
}

/// One line of the activity feed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Event identifier.
    pub id: EventId,
    /// What happened.
    pub kind: EventKind,
    /// Who did it. Absent once the account is deleted.
    pub actor: Option<UserRef>,
    /// Repository it happened to. Absent for account-level events.
    pub repository: Option<Slug>,
    /// When it happened.
    pub created_at: Timestamp,
}
