//! A string that must never reach a log line or an API response.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Wraps a value whose contents are withheld from `Debug` and serialization.
///
/// The database URL carries a password, and a personal access token is only ever
/// shown to the user who created it, so both travel through the code inside this
/// type and only leave it at the point of use.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    /// Wrap a value.
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Read the wrapped value. Every call site is a place worth reviewing.
    pub const fn expose(&self) -> &T {
        &self.0
    }

    /// Unwrap the value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(<redacted>)")
    }
}

impl<T> fmt::Display for Secret<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: Serialize> Serialize for Secret<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("<redacted>")
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Secret<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_and_display_withhold_the_value() {
        let secret = Secret::new("postgres://user:hunter2@localhost/scabby".to_owned());
        assert_eq!(format!("{secret:?}"), "Secret(<redacted>)");
        assert_eq!(format!("{secret}"), "<redacted>");
        assert!(secret.expose().contains("hunter2"));
    }
}
