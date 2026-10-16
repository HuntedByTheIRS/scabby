//! Repositories and who can see them.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::ids::RepositoryId;
use crate::names::{Name, Slug};
use crate::timestamp::Timestamp;
use crate::user::UserRef;

/// Who can read a repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    /// Anyone, signed in or not.
    Public,
    /// Only the owner.
    Private,
}

impl Visibility {
    /// The lowercase form used in the database and the API.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Private => "private",
        }
    }

    /// Parse the stored form.
    pub fn parse(raw: &str) -> Result<Self> {
        match raw {
            "public" => Ok(Self::Public),
            "private" => Ok(Self::Private),
            other => Err(Error::validation(
                "visibility",
                format!("unknown visibility {other}"),
            )),
        }
    }

    /// Whether this hides the repository from anonymous readers.
    pub const fn is_private(self) -> bool {
        matches!(self, Self::Private)
    }
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Visibility {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self> {
        Self::parse(raw)
    }
}

/// A repository as stored in the database.
///
/// `clone_url` and `html_url` are built by the web layer, which is the only place that knows the
/// instance's base URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository identifier.
    pub id: RepositoryId,
    /// Account that owns it.
    pub owner: UserRef,
    /// Name without the `.git` suffix.
    pub name: Name,
    /// One-line description.
    pub description: Option<String>,
    /// Who can read it.
    pub visibility: Visibility,
    /// Branch checked out on clone.
    pub default_branch: String,
    /// Size of the bare repository on disk, as of the last write.
    pub size_bytes: i64,
    /// Issues currently open, for list rows.
    pub open_issue_count: i64,
    /// When it was created.
    pub created_at: Timestamp,
    /// When it last changed.
    pub updated_at: Timestamp,
}

impl Repository {
    /// The `owner/name` pair this repository is addressed by.
    pub fn slug(&self) -> Slug {
        Slug::new(self.owner.username.clone(), self.name.clone())
    }

    /// `owner/name`, the form the API and the UI show.
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner.username, self.name)
    }
}
