//! Instance configuration, read from a TOML file and overridden by flags.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{Error, Result};
use crate::secret::Secret;

/// Address the server binds when nothing overrides it.
pub const DEFAULT_LISTEN: &str = "127.0.0.1:3000";

/// Database the server connects to when nothing overrides it.
pub const DEFAULT_DATABASE_URL: &str = "postgres://scabby:scabby@localhost:5432/scabby";

/// Directory holding bare repositories when nothing overrides it.
pub const DEFAULT_DATA_DIR: &str = "data";

/// Public origin used to build clone URLs and links when nothing overrides it.
pub const DEFAULT_BASE_URL: &str = "http://localhost:3000";

/// Whether the instance lets people sign themselves up.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Registration {
    /// Anyone may create an account.
    #[default]
    Open,
    /// Accounts are created by an administrator with `scabby admin create-user`.
    Closed,
}

impl Registration {
    /// The lowercase form used in the config file.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Closed => "closed",
        }
    }

    /// Whether sign-up is allowed.
    pub const fn is_open(self) -> bool {
        matches!(self, Self::Open)
    }
}

impl std::fmt::Display for Registration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Registration {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self> {
        match raw.to_ascii_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            other => Err(Error::validation(
                "registration",
                format!("unknown mode {other}"),
            )),
        }
    }
}

/// Everything the server needs to start.
///
/// Values arrive from three places, lowest precedence first: this file's defaults, a TOML file
/// named by `--config`, then command-line flags. Every flag also reads a `SCABBY_`-prefixed
/// environment variable, so a container needs no arguments at all.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Address the HTTP server binds.
    pub listen: SocketAddr,
    /// Postgres connection string. Withheld from `Debug` and serialization.
    pub database_url: Secret<String>,
    /// Root directory for bare repositories.
    pub data_dir: PathBuf,
    /// Public origin, used for clone URLs, cookies, and links.
    pub base_url: Url,
    /// Serve the UI from this directory instead of the embedded copy.
    pub frontend_dir: Option<PathBuf>,
    /// Whether sign-up is allowed.
    pub registration: Registration,
    /// Ceiling on database connections.
    pub max_connections: u32,
    /// Run migrations on startup.
    pub migrate_on_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: DEFAULT_LISTEN
                .parse()
                .expect("default listen address is valid"),
            database_url: Secret::new(DEFAULT_DATABASE_URL.to_owned()),
            data_dir: PathBuf::from(DEFAULT_DATA_DIR),
            base_url: DEFAULT_BASE_URL.parse().expect("default base URL is valid"),
            frontend_dir: None,
            registration: Registration::default(),
            max_connections: 10,
            migrate_on_start: true,
        }
    }
}

impl Config {
    /// Parse a TOML config file.
    ///
    /// Unknown keys are an error rather than a silent no-op, so a typo in a deployment file is
    /// caught at startup instead of at the moment someone notices the setting never took.
    pub fn from_toml(text: &str) -> Result<Self> {
        toml::from_str(text).map_err(|error| Error::validation("config", error.to_string()))
    }

    /// Read a config file, returning the defaults when the path is absent or the file is missing.
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        if !path.exists() {
            return Err(Error::not_found("config file", path.display()));
        }
        let text = std::fs::read_to_string(path)
            .map_err(|error| Error::internal(format!("{}: {error}", path.display())))?;
        Self::from_toml(&text)
    }

    /// Reject combinations that would fail later, at the point where the message can name the fix.
    pub fn validate(&self) -> Result<()> {
        if !matches!(self.base_url.scheme(), "http" | "https") {
            return Err(Error::validation("base_url", "must use http or https"));
        }
        if self.data_dir.as_os_str().is_empty() {
            return Err(Error::validation("data_dir", "must not be empty"));
        }
        if self.max_connections == 0 {
            return Err(Error::validation("max_connections", "must be at least 1"));
        }
        if self.database_url.expose().is_empty() {
            return Err(Error::validation("database_url", "must not be empty"));
        }
        Ok(())
    }

    /// The database URL, at the one point where it has to be unwrapped.
    pub fn database_url(&self) -> &str {
        self.database_url.expose()
    }

    /// Trailing-slash-free origin, used to build clone URLs.
    pub fn origin(&self) -> String {
        self.base_url.as_str().trim_end_matches('/').to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_files_keep_the_defaults() {
        let config =
            Config::from_toml("listen = \"0.0.0.0:8080\"\nregistration = \"closed\"\n").unwrap();
        assert_eq!(config.listen.port(), 8080);
        assert_eq!(config.registration, Registration::Closed);
        assert_eq!(config.data_dir, PathBuf::from(DEFAULT_DATA_DIR));
        assert_eq!(config.max_connections, 10);
    }

    #[test]
    fn unknown_keys_are_rejected() {
        let error = Config::from_toml("databse_url = \"oops\"\n").unwrap_err();
        assert!(error.to_string().contains("databse_url"));
    }

    #[test]
    fn debug_output_withholds_the_database_url() {
        let config = Config::default();
        let rendered = format!("{config:?}");
        assert!(rendered.contains("<redacted>"));
        assert!(!rendered.contains("scabby:scabby"));
    }
}
