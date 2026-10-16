//! Git objects as the API and the web UI see them.

use serde::{Deserialize, Serialize};

use crate::ids::Sha;
use crate::timestamp::Timestamp;

/// Who wrote or committed a change, as recorded in the commit itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// Name from the commit header.
    pub name: String,
    /// Email from the commit header.
    pub email: String,
    /// When the signature was made.
    pub when: Timestamp,
}

/// A commit without its diff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitSummary {
    /// Full object id.
    pub sha: Sha,
    /// First line of the commit message.
    pub message: String,
    /// Who authored the change.
    pub author: Signature,
    /// Who recorded the change.
    pub committer: Signature,
    /// Parent object ids, empty for a root commit.
    pub parents: Vec<Sha>,
    /// Author date, repeated so a list can sort without unpacking `author`.
    pub when: Timestamp,
}

impl CommitSummary {
    /// The abbreviation used in lists and URLs.
    pub fn short_sha(&self) -> String {
        self.sha.short().to_owned()
    }
}

/// How a file changed between a commit and its first parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeStatus {
    /// The file is new.
    Added,
    /// The file changed in place.
    Modified,
    /// The file is gone.
    Deleted,
    /// The file moved, possibly with edits.
    Renamed,
}

/// One entry of a commit's file list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    /// Path within the repository.
    pub path: String,
    /// What happened to it.
    pub status: ChangeStatus,
    /// Lines added.
    pub additions: u64,
    /// Lines removed.
    pub deletions: u64,
    /// True when the content is not text, in which case the counts are zero.
    pub binary: bool,
}

/// Totals for a commit's change set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffStats {
    /// Lines added across every file.
    pub additions: u64,
    /// Lines removed across every file.
    pub deletions: u64,
    /// Number of files touched.
    pub files: u64,
}

/// A commit with its message body and change set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitDetail {
    /// The commit itself.
    #[serde(flatten)]
    pub summary: CommitSummary,
    /// Everything after the first line of the message.
    pub body: String,
    /// Files touched, against the first parent.
    pub files: Vec<FileChange>,
    /// Totals for the files above.
    pub stats: DiffStats,
}

/// What a tree entry points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntryKind {
    /// A regular file.
    File,
    /// A directory.
    Dir,
    /// A symbolic link.
    Symlink,
    /// A gitlink to another repository.
    Submodule,
}

/// One entry in a directory listing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    /// What the entry is, serialized as `type`.
    #[serde(rename = "type")]
    pub kind: EntryKind,
    /// Last path component.
    pub name: String,
    /// Full path from the repository root.
    pub path: String,
    /// Size in bytes, absent for directories.
    pub size: Option<u64>,
}

/// A directory at a revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeListing {
    /// Path listed, empty for the root.
    pub path: String,
    /// Entries in git's own order: directories sorted with a trailing slash.
    pub entries: Vec<TreeEntry>,
}

/// A file at a revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blob {
    /// Last path component.
    pub name: String,
    /// Full path from the repository root.
    pub path: String,
    /// Size in bytes.
    pub size: u64,
    /// Object id of the blob.
    pub sha: Sha,
    /// True when the content is not text.
    pub binary: bool,
    /// Decoded contents when the file is text.
    pub text: Option<String>,
    /// Base64 contents, always present, because the UI cannot always fetch the raw URL.
    pub content: String,
}

/// A branch and the commit it points at.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Branch name without `refs/heads/`.
    pub name: String,
    /// Commit the branch points at.
    pub sha: Sha,
    /// Whether this is the repository's default branch.
    pub is_default: bool,
    /// The commit, when it could be read.
    pub commit: Option<CommitSummary>,
}

/// A tag and the commit it points at.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag name without `refs/tags/`.
    pub name: String,
    /// Object the tag points at, which for an annotated tag is the tag object.
    pub sha: Sha,
    /// The commit the tag resolves to, when it resolves.
    pub commit: Option<CommitSummary>,
}

/// Counts and the head commit for a repository on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStats {
    /// Number of local branches.
    pub branches: usize,
    /// Number of tags.
    pub tags: usize,
    /// Bytes the bare repository occupies.
    pub disk_bytes: u64,
    /// Commits reachable from the default branch, when it can be walked.
    pub commit_count: Option<u64>,
    /// Tip of the default branch.
    pub head_commit: Option<CommitSummary>,
}
