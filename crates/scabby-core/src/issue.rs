//! Issues and their comments.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::ids::{IssueCommentId, IssueId};
use crate::timestamp::Timestamp;
use crate::user::UserRef;

/// Whether an issue is still open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    /// Being worked on, or at least not closed.
    Open,
    /// Closed, and could be reopened.
    Closed,
}

impl IssueState {
    /// The lowercase form used in the database and the API.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }

    /// Parse the stored form.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            other => Err(Error::validation(
                "state",
                format!("unknown issue state {other}"),
            )),
        }
    }
}

impl fmt::Display for IssueState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for IssueState {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self> {
        Self::parse(raw)
    }
}

/// Filter used by the issue list endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueFilter {
    /// Only open issues, the default.
    Open,
    /// Only closed issues.
    Closed,
    /// Both.
    All,
}

impl IssueFilter {
    /// Parse the `state` query parameter.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            "all" => Ok(Self::All),
            other => Err(Error::validation(
                "state",
                format!("unknown state filter {other}"),
            )),
        }
    }

    /// The stored state this filter selects, or `None` for [`IssueFilter::All`].
    pub const fn as_state(self) -> Option<IssueState> {
        match self {
            Self::Open => Some(IssueState::Open),
            Self::Closed => Some(IssueState::Closed),
            Self::All => None,
        }
    }
}

/// An issue, numbered within its repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue identifier.
    pub id: IssueId,
    /// Per-repository number, used in URLs.
    pub number: i32,
    /// Issue title.
    pub title: String,
    /// Opening message.
    pub body: Option<String>,
    /// Open or closed.
    pub state: IssueState,
    /// Who opened it.
    pub author: UserRef,
    /// Number of comments, for list rows.
    pub comment_count: i64,
    /// When it was opened.
    pub created_at: Timestamp,
    /// When it last changed.
    pub updated_at: Timestamp,
    /// When it was closed, if it is closed.
    pub closed_at: Option<Timestamp>,
}

/// A comment on an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    /// Comment identifier.
    pub id: IssueCommentId,
    /// Number of the issue it belongs to.
    pub issue_number: i32,
    /// Who wrote it.
    pub author: UserRef,
    /// Markdown body.
    pub body: String,
    /// When it was written.
    pub created_at: Timestamp,
    /// When it was last edited.
    pub updated_at: Timestamp,
}
