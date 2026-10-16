//! Personal access tokens and SSH public keys.

use serde::{Deserialize, Serialize};

use crate::ids::{SshKeyId, TokenId};
use crate::timestamp::Timestamp;

/// Prefix every personal access token carries, so a leaked one is recognizable.
pub const TOKEN_PREFIX: &str = "scabby_pat_";

/// Characters of a token kept in clear text for display: the prefix plus six.
pub const TOKEN_DISPLAY_LEN: usize = 17;

/// A personal access token. The secret itself is never part of this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    /// Token identifier.
    pub id: TokenId,
    /// Label the user gave it, such as `ci`.
    pub name: String,
    /// Recognizable head of the token, for the settings list.
    pub prefix: String,
    /// Scopes recorded at creation. Not enforced yet.
    pub scopes: Vec<String>,
    /// When it was created.
    pub created_at: Timestamp,
    /// When it was last used to authenticate.
    pub last_used_at: Option<Timestamp>,
    /// When it stops working, if it expires.
    pub expires_at: Option<Timestamp>,
}

/// An SSH public key registered by a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKey {
    /// Key identifier.
    pub id: SshKeyId,
    /// Label the user gave it, such as `laptop`.
    pub title: String,
    /// The key as pasted, one line of OpenSSH text.
    pub public_key: String,
    /// `SHA256:` fingerprint, the form OpenSSH prints.
    pub fingerprint: String,
    /// When it was added.
    pub created_at: Timestamp,
    /// When it was last used.
    pub last_used_at: Option<Timestamp>,
}
