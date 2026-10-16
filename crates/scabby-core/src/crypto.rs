//! Password hashing, token generation, and key fingerprints.
//!
//! Three secrets pass through here and none of them is stored in the clear. Passwords become
//! argon2id hashes salted per password; session tokens and personal access tokens become SHA-256
//! digests, which is enough because the value going in is already 256 random bits. The functions
//! live in this crate because two callers need them: the login endpoint and `scabby admin
//! create-user`. A second implementation of password hashing is how two halves of a program end up
//! disagreeing about how a password is checked.

use argon2::Argon2;
// The traits are all that is named: `hash_password` produces an owned PHC value and
// `verify_password` parses the stored string itself, so neither type needs importing.
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, STANDARD_NO_PAD};
use rand::RngExt as _;
use sha2::{Digest, Sha256};

use crate::auth::{TOKEN_DISPLAY_LEN, TOKEN_PREFIX};
use crate::error::{Error, Result};

/// Prefix of an argon2 hash, checked before comparing, so a column that holds something else is
/// treated as a mismatch rather than being fed to the parser.
pub const ARGON2_PREFIX: &str = "$argon2";

/// Shortest password Scabby accepts, anywhere it accepts one.
pub const MIN_PASSWORD_LEN: usize = 8;

/// Random bytes from the operating system.
pub fn random_bytes(count: usize) -> Vec<u8> {
    let mut bytes = vec![0_u8; count];
    rand::rng().fill(&mut bytes[..]);
    bytes
}

/// Lowercase hex, the form tokens and identifiers are stored and compared in.
pub fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push_str(&format!("{byte:02x}"));
    }
    text
}

/// Hash a password for storage.
///
/// Argon2 generates the salt, so nothing here picks one or reuses one: two calls with the same
/// password produce two different hashes, which is the property that makes a stolen table useless
/// against a wordlist.
pub fn hash_password(password: &str) -> Result<String> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|error| Error::internal(format!("could not hash a password: {error}")))
}

/// Whether a password matches a stored hash.
///
/// A malformed hash is a mismatch rather than an error: the caller cannot tell the two apart, which
/// is the point. An account whose hash column was corrupted does not become an account that anyone
/// can sign into.
pub fn verify_password(hash: &str, password: &str) -> bool {
    if !hash.starts_with(ARGON2_PREFIX) {
        return false;
    }

    Argon2::default()
        .verify_password(password.as_bytes(), hash)
        .is_ok()
}

/// A fresh session token: 32 random bytes, hex encoded.
pub fn new_session_token() -> String {
    hex(&random_bytes(32))
}

/// A fresh personal access token, carrying the prefix that makes a leak recognizable in a log.
pub fn new_api_token() -> String {
    format!("{TOKEN_PREFIX}{}", hex(&random_bytes(24)))
}

/// The digest stored for a session or an access token.
pub fn hash_token(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// The recognizable head of a token, kept in clear text so the settings list can name one.
pub fn token_prefix(token: &str) -> String {
    token.chars().take(TOKEN_DISPLAY_LEN).collect()
}

/// The OpenSSH fingerprint of a public key, in the `SHA256:...` form OpenSSH prints.
///
/// An `authorized_keys` line is an algorithm name, a base64 blob, and an optional comment. Hashing
/// the decoded blob is what `ssh-keygen -lf` does, so a user can run that command and compare the
/// two values, which is the only way a fingerprint is worth anything.
pub fn ssh_fingerprint(public_key: &str) -> Result<String> {
    /// Key types Scabby will store. Anything else is refused rather than stored unread.
    const KNOWN_TYPES: &[&str] = &[
        "ssh-ed25519",
        "ssh-rsa",
        "ecdsa-sha2-nistp256",
        "sk-ssh-ed25519@openssh.com",
    ];

    let mut fields = public_key.split_whitespace();
    let kind = fields.next().unwrap_or_default();
    if !KNOWN_TYPES.contains(&kind) {
        return Err(Error::validation("key", "must be an OpenSSH public key"));
    }

    let blob = fields
        .next()
        .ok_or_else(|| Error::validation("key", "must be an OpenSSH public key"))?;
    let decoded = STANDARD
        .decode(blob)
        .map_err(|_| Error::validation("key", "is not valid base64"))?;

    Ok(format!(
        "SHA256:{}",
        STANDARD_NO_PAD.encode(Sha256::digest(&decoded))
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwords_hash_and_verify() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(hash.starts_with(ARGON2_PREFIX));
        assert!(verify_password(&hash, "correct horse battery staple"));
        assert!(!verify_password(&hash, "wrong"));
        assert!(
            !verify_password("not-a-hash", "wrong"),
            "a malformed hash must be a mismatch"
        );
    }

    #[test]
    fn the_same_password_hashes_differently_every_time() {
        let first = hash_password("same password").unwrap();
        let second = hash_password("same password").unwrap();
        assert_ne!(first, second, "each hash carries its own salt");
    }

    #[test]
    fn tokens_are_prefixed_and_hashed_consistently() {
        let token = new_api_token();
        assert!(token.starts_with(TOKEN_PREFIX));
        assert_eq!(token_prefix(&token).len(), TOKEN_DISPLAY_LEN);
        assert_eq!(hash_token(&token), hash_token(&token));
        assert_ne!(hash_token(&token), hash_token(&new_api_token()));
        assert_ne!(new_session_token(), new_session_token());
    }

    #[test]
    fn fingerprints_hash_the_decoded_key_blob() {
        // A syntactically valid ed25519 line. The fingerprint is whatever OpenSSH computes from the
        // same decoded blob, which is why the blob is hashed rather than the text around it.
        let key = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJ1yZ3VybmtleWV4YW1wbGVrZXlmb3J0ZXN0aW5n user@host";
        let fingerprint = ssh_fingerprint(key).unwrap();
        assert!(fingerprint.starts_with("SHA256:"));
        assert!(
            !fingerprint.contains('='),
            "openssh prints the fingerprint without padding"
        );
        assert_eq!(
            fingerprint,
            ssh_fingerprint(&format!("{key}  \n")).unwrap(),
            "a comment is ignored"
        );

        assert!(ssh_fingerprint("not a key").is_err());
        assert!(ssh_fingerprint("ssh-ed25519 !!!notbase64!!! comment").is_err());
        assert!(
            ssh_fingerprint("ssh-dss AAAAB3NzaC1kc3MAAACB").is_err(),
            "unsupported key type"
        );
    }
}
