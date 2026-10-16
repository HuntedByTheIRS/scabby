//! A UTC timestamp with one wire format.

use std::fmt;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::error::{Error, Result};

/// A point in time, serialized as RFC 3339 in UTC everywhere it appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Timestamp(OffsetDateTime);

impl Timestamp {
    /// The current time.
    pub fn now() -> Self {
        Self(OffsetDateTime::now_utc())
    }

    /// Wrap a value read from the database.
    pub const fn from_offset(value: OffsetDateTime) -> Self {
        Self(value)
    }

    /// The underlying value, for storage and arithmetic.
    pub const fn as_offset(self) -> OffsetDateTime {
        self.0
    }

    /// RFC 3339 text, the only form this type crosses a process boundary in.
    pub fn to_rfc3339(self) -> String {
        self.0.format(&Rfc3339).unwrap_or_else(|_| String::new())
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_rfc3339())
    }
}

impl From<OffsetDateTime> for Timestamp {
    fn from(value: OffsetDateTime) -> Self {
        Self(value)
    }
}

impl From<Timestamp> for OffsetDateTime {
    fn from(value: Timestamp) -> Self {
        value.0
    }
}

impl TryFrom<String> for Timestamp {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        OffsetDateTime::parse(&value, &Rfc3339)
            .map(Self)
            .map_err(|_| Error::validation("timestamp", "must be an RFC 3339 date-time"))
    }
}

impl From<Timestamp> for String {
    fn from(value: Timestamp) -> Self {
        value.to_rfc3339()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_rfc3339() {
        let parsed: Timestamp = "2026-10-15T18:00:00Z".to_owned().try_into().unwrap();
        assert_eq!(parsed.to_rfc3339(), "2026-10-15T18:00:00Z");
        assert!(Timestamp::try_from("yesterday".to_owned()).is_err());
    }
}
