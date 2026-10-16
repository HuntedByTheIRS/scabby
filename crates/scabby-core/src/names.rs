//! Validation for the names that appear in URLs and on disk.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// Longest owner or repository name Scabby accepts.
pub const MAX_NAME_LEN: usize = 100;

/// Names the router claims for itself, so a user cannot take one.
const RESERVED: &[&str] = &[
    "api",
    "assets",
    "dashboard",
    "healthz",
    "login",
    "new",
    "register",
    "repos",
    "robots.txt",
    "favicon.ico",
    "settings",
    "users",
];

/// An owner or repository name: ASCII letters, digits, `.`, `_`, and `-`.
///
/// A name has to be usable as a single path segment and as a directory name, so
/// it cannot begin with a dot or a dash, cannot contain `..`, and cannot end in
/// `.git` (the on-disk directory appends that suffix itself).
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Name(String);

impl Name {
    /// Validate and wrap a name.
    pub fn parse(raw: &str) -> Result<Self> {
        let invalid = |message: &str| Error::validation("name", message.to_owned());

        if raw.is_empty() {
            return Err(invalid("must not be empty"));
        }
        if raw.len() > MAX_NAME_LEN {
            return Err(invalid(&format!(
                "must be at most {MAX_NAME_LEN} characters"
            )));
        }
        if !raw
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
        {
            return Err(invalid(
                "may only contain letters, digits, '.', '_', and '-'",
            ));
        }
        if !raw.starts_with(|first: char| first.is_ascii_alphanumeric()) {
            return Err(invalid("must start with a letter or digit"));
        }
        if raw.ends_with('.') || raw.contains("..") {
            return Err(invalid("must not contain '..' or end with '.'"));
        }
        if raw.to_ascii_lowercase().ends_with(".git") {
            return Err(invalid("must not end in '.git'"));
        }
        let lowered = raw.to_ascii_lowercase();
        if RESERVED.contains(&lowered.as_str()) {
            return Err(invalid("is reserved by the server"));
        }
        Ok(Self(raw.to_owned()))
    }

    /// The name as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The name lowercased, which is how uniqueness is decided.
    pub fn lowered(&self) -> String {
        self.0.to_ascii_lowercase()
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Name({})", self.0)
    }
}

impl FromStr for Name {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self> {
        Self::parse(raw)
    }
}

impl TryFrom<String> for Name {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<Name> for String {
    fn from(value: Name) -> Self {
        value.0
    }
}

/// The `owner/name` pair that identifies a repository.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Slug {
    /// Account that owns the repository.
    pub owner: Name,
    /// Repository name without the `.git` suffix.
    pub name: Name,
}

impl Slug {
    /// Pair an owner and a repository name.
    pub const fn new(owner: Name, name: Name) -> Self {
        Self { owner, name }
    }

    /// Parse `owner/name`.
    pub fn parse(raw: &str) -> Result<Self> {
        let (owner, name) = raw
            .split_once('/')
            .ok_or_else(|| Error::validation("slug", "must look like owner/name"))?;
        if name.contains('/') {
            return Err(Error::validation("slug", "must look like owner/name"));
        }
        Ok(Self {
            owner: Name::parse(owner)?,
            name: Name::parse(name)?,
        })
    }

    /// `owner/name`, as it appears in URLs and the API.
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.name)
    }

    /// Path of the bare repository relative to the git root, `owner/name.git`.
    pub fn relative_path(&self) -> PathBuf {
        PathBuf::from(self.owner.as_str()).join(format!("{}.git", self.name))
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.owner, self.name)
    }
}

impl FromStr for Slug {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self> {
        Self::parse(raw)
    }
}

impl TryFrom<String> for Slug {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Self::parse(&value)
    }
}

impl From<Slug> for String {
    fn from(value: Slug) -> Self {
        value.full_name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ordinary_names() {
        for raw in ["ada", "scabby", "scabby-web", "v0.1", "a_b", "A1"] {
            assert_eq!(Name::parse(raw).unwrap().as_str(), raw, "{raw}");
        }
    }

    #[test]
    fn rejects_names_that_break_paths_or_routes() {
        for raw in [
            "",
            ".hidden",
            "-dash",
            "trailing.",
            "a..b",
            "a/b",
            "a b",
            "repo.git",
            "api",
        ] {
            assert!(Name::parse(raw).is_err(), "{raw} should be rejected");
        }
    }

    #[test]
    fn reserved_names_are_case_insensitive() {
        assert!(Name::parse("Settings").is_err());
    }

    #[test]
    fn slug_round_trips_and_maps_to_a_directory() {
        let slug = Slug::parse("ada/scabby").unwrap();
        assert_eq!(slug.to_string(), "ada/scabby");
        assert_eq!(slug.relative_path(), PathBuf::from("ada/scabby.git"));
        assert!(Slug::parse("ada").is_err());
        assert!(Slug::parse("ada/scabby/extra").is_err());
    }
}
