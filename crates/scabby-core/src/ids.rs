//! Identifiers for stored objects, and the git object id.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

macro_rules! id_type {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(i64);

        impl $name {
            /// Wrap a value read from the database.
            pub const fn new(value: i64) -> Self {
                Self(value)
            }

            /// The underlying value.
            pub const fn get(self) -> i64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<i64> for $name {
            fn from(value: i64) -> Self {
                Self(value)
            }
        }
    };
}

id_type!(UserId, "A user account identifier.");
id_type!(RepositoryId, "A repository identifier.");
id_type!(IssueId, "An issue identifier.");
id_type!(IssueCommentId, "An issue comment identifier.");
id_type!(TokenId, "A personal access token identifier.");
id_type!(SshKeyId, "An SSH public key identifier.");
id_type!(EventId, "An activity feed entry identifier.");

/// Number of hex characters in a full git object id.
pub const SHA_LEN: usize = 40;

/// Shortest abbreviation accepted where a revision is read from a URL.
pub const MIN_SHA_LEN: usize = 4;

/// A git object id.
///
/// Git accepts unambiguous abbreviations, so a value shorter than [`SHA_LEN`] is
/// kept as written and resolved by the git layer. Comparison and display use the
/// text exactly as it was parsed, lowercased.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Sha(String);

impl Sha {
    /// Validate an object id or an abbreviation of one.
    pub fn parse(raw: &str) -> Result<Self> {
        let trimmed = raw.trim();
        if !(MIN_SHA_LEN..=SHA_LEN).contains(&trimmed.len()) {
            return Err(Error::validation(
                "sha",
                format!("must be between {MIN_SHA_LEN} and {SHA_LEN} hex characters"),
            ));
        }
        if !trimmed.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(Error::validation("sha", "must be hexadecimal"));
        }
        Ok(Self(trimmed.to_ascii_lowercase()))
    }

    /// The object id as text.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The abbreviation used in lists and URLs, seven characters when available.
    pub fn short(&self) -> &str {
        let end = self.0.len().min(7);
        &self.0[..end]
    }

    /// Whether this is a full object id rather than an abbreviation.
    pub fn is_full(&self) -> bool {
        self.0.len() == SHA_LEN
    }
}

impl fmt::Display for Sha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Sha {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self> {
        Self::parse(raw)
    }
}

impl TryFrom<String> for Sha {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<Sha> for String {
    fn from(value: Sha) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = "8a5f3c1b7d2e4f6079a1b2c3d4e5f60718293a4b";

    #[test]
    fn accepts_full_and_abbreviated_ids() {
        assert!(Sha::parse(FULL).unwrap().is_full());
        assert_eq!(Sha::parse("8a5f3c1").unwrap().short(), "8a5f3c1");
        assert_eq!(Sha::parse("8A5F3C1").unwrap().as_str(), "8a5f3c1");
    }

    #[test]
    fn rejects_junk() {
        assert!(Sha::parse("").is_err());
        assert!(Sha::parse("abc").is_err());
        assert!(Sha::parse("zzzzzzzz").is_err());
        assert!(Sha::parse(&"a".repeat(SHA_LEN + 1)).is_err());
    }
}
