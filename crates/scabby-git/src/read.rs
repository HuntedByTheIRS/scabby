//! Reading commits, trees, blobs, and refs with gix.

use scabby_core::git::{
    Blob, Branch, CommitDetail, CommitSummary, DiffStats, EntryKind, Signature, Tag, TreeEntry,
    TreeListing,
};
use scabby_core::ids::Sha;
use scabby_core::timestamp::Timestamp;
use time::{OffsetDateTime, UtcOffset};

use crate::Result;
use crate::error::GitError;

/// How many commits a stats walk will count before reporting the number as unknown.
pub const COMMIT_COUNT_LIMIT: usize = 100_000;

/// Largest blob the API decodes as text; anything above it is reported as binary content only.
pub const MAX_TEXT_BYTES: usize = 1024 * 1024;

/// File names a repository's landing page will show, in the order they are preferred.
const README_CANDIDATES: &[&str] = &[
    "README.md",
    "README.markdown",
    "README.rst",
    "README.txt",
    "README",
    "readme.md",
];

/// Resolve a revision string, which may be a branch, a tag, or an abbreviated object id.
pub fn resolve(repo: &gix::Repository, revision: &str) -> Result<Sha> {
    let id = repo
        .rev_parse_single(revision)
        .map_err(|_| GitError::UnknownRevision {
            revision: revision.to_owned(),
        })?;
    Ok(Sha::parse(&id.detach().to_string())?)
}

/// The commit a revision points at.
pub fn commit(repo: &gix::Repository, revision: &str) -> Result<CommitSummary> {
    let id = repo
        .rev_parse_single(revision)
        .map_err(|_| GitError::UnknownRevision {
            revision: revision.to_owned(),
        })?;

    let object = id.object().map_err(GitError::from_git)?;
    let commit = object
        .try_into_commit()
        .map_err(|_| GitError::UnknownRevision {
            revision: revision.to_owned(),
        })?;

    summary(&commit)
}

/// The commit a revision points at, or `None` when the repository has no such revision yet.
pub fn try_commit(repo: &gix::Repository, revision: &str) -> Result<Option<CommitSummary>> {
    match commit(repo, revision) {
        Ok(summary) => Ok(Some(summary)),
        Err(GitError::UnknownRevision { .. }) => Ok(None),
        Err(other) => Err(other),
    }
}

/// A commit with its message body, so the caller can add the file list from the diff layer.
pub fn commit_detail(repo: &gix::Repository, revision: &str) -> Result<(CommitSummary, String)> {
    let id = repo
        .rev_parse_single(revision)
        .map_err(|_| GitError::UnknownRevision {
            revision: revision.to_owned(),
        })?;
    let object = id.object().map_err(GitError::from_git)?;
    let commit = object
        .try_into_commit()
        .map_err(|_| GitError::UnknownRevision {
            revision: revision.to_owned(),
        })?;

    let summary = summary(&commit)?;
    let message = commit.message().map_err(GitError::from_git)?;
    let body = message
        .body
        .map(|body| body.to_string())
        .unwrap_or_default();

    Ok((summary, body))
}

/// The tip of a branch, or of `HEAD` when there is no branch yet.
pub fn head_commit(repo: &gix::Repository, default_branch: &str) -> Result<Option<CommitSummary>> {
    try_commit(repo, default_branch)
}

/// Commits reachable from a revision, newest first.
pub fn log(
    repo: &gix::Repository,
    revision: &str,
    skip: usize,
    limit: usize,
) -> Result<Vec<CommitSummary>> {
    let id = repo
        .rev_parse_single(revision)
        .map_err(|_| GitError::UnknownRevision {
            revision: revision.to_owned(),
        })?;

    let walk = id
        .ancestors()
        .sorting(gix::revision::walk::Sorting::ByCommitTime(
            gix::traverse::commit::simple::CommitTimeOrder::NewestFirst,
        ))
        .all()
        .map_err(GitError::from_git)?;

    let mut commits = Vec::with_capacity(limit.min(256));
    for info in walk.skip(skip).take(limit) {
        let info = info.map_err(GitError::from_git)?;
        let commit = info.object().map_err(GitError::from_git)?;
        commits.push(summary(&commit)?);
    }
    Ok(commits)
}

/// How many commits are reachable from a revision, or `None` past the cap.
///
/// The cap is not a limitation to hide: a shallow count reported as a total would be a lie, so the
/// API gets `null` and can say the history is larger than the tree was willing to walk.
pub fn commit_count(repo: &gix::Repository, revision: &str, cap: usize) -> Result<Option<u64>> {
    let Ok(id) = repo.rev_parse_single(revision) else {
        return Ok(None);
    };

    let walk = id
        .ancestors()
        .sorting(gix::revision::walk::Sorting::BreadthFirst)
        .all()
        .map_err(GitError::from_git)?;

    let mut count = 0_u64;
    for info in walk {
        info.map_err(GitError::from_git)?;
        count += 1;
        if count as usize > cap {
            return Ok(None);
        }
    }
    Ok(Some(count))
}

/// Local branches, with the commit each one points at.
pub fn branches(repo: &gix::Repository, default_branch: &str) -> Result<Vec<Branch>> {
    let references = repo.references().map_err(GitError::from_git)?;
    let local = references.local_branches().map_err(GitError::from_git)?;

    let mut branches = Vec::new();
    for reference in local {
        let reference = reference.map_err(GitError::from_git)?;
        let name = reference.name().shorten().to_string();
        let sha = Sha::parse(&reference.id().detach().to_string())?;
        let is_default = name == default_branch;
        let commit = try_commit(repo, &name)?;

        branches.push(Branch {
            name,
            sha,
            is_default,
            commit,
        });
    }

    // The default branch first, then alphabetically, which is the order the UI shows them in.
    branches.sort_by(|a, b| {
        b.is_default
            .cmp(&a.is_default)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(branches)
}

/// Tags, with the commit each one resolves to.
pub fn tags(repo: &gix::Repository) -> Result<Vec<Tag>> {
    let references = repo.references().map_err(GitError::from_git)?;
    let tags = references.tags().map_err(GitError::from_git)?;

    let mut collected = Vec::new();
    for reference in tags {
        let reference = reference.map_err(GitError::from_git)?;
        let name = reference.name().shorten().to_string();
        let sha = Sha::parse(&reference.id().detach().to_string())?;

        // An annotated tag points at a tag object, so resolve it through the peel to reach a commit.
        let commit = match reference.into_fully_peeled_id() {
            Ok(id) => match id.object() {
                Ok(object) => object
                    .try_into_commit()
                    .ok()
                    .and_then(|commit| summary(&commit).ok()),
                Err(_) => None,
            },
            Err(_) => None,
        };

        collected.push(Tag { name, sha, commit });
    }

    collected.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(collected)
}

/// A directory at a revision.
pub fn tree(repo: &gix::Repository, revision: &str, path: &str) -> Result<TreeListing> {
    let commit = peeled_commit(repo, revision)?;
    let mut tree = commit.tree().map_err(GitError::from_git)?;

    // A path that is not a directory is reported as missing rather than as an empty listing, so the
    // caller can fall back to reading it as a file.
    if !path.is_empty() {
        let entry = tree
            .peel_to_entry_by_path(path)
            .map_err(GitError::from_git)?
            .ok_or_else(|| GitError::MissingPath {
                path: path.to_owned(),
                revision: revision.to_owned(),
            })?;

        if !entry.mode().is_tree() {
            return Err(GitError::MissingPath {
                path: path.to_owned(),
                revision: revision.to_owned(),
            });
        }

        let id = entry.id();
        let object = id.object().map_err(GitError::from_git)?;
        tree = object.try_into_tree().map_err(|_| GitError::MissingPath {
            path: path.to_owned(),
            revision: revision.to_owned(),
        })?;
    }

    let resolved = path.to_owned();
    let mut listing = Vec::new();
    for entry in tree.iter() {
        let entry = entry.map_err(GitError::from_git)?;
        let name = entry.filename().to_string();
        let entry_path = if resolved.is_empty() {
            name.clone()
        } else {
            format!("{resolved}/{name}")
        };

        let mode = entry.mode();
        let kind = if mode.is_tree() {
            EntryKind::Dir
        } else if mode.is_link() {
            EntryKind::Symlink
        } else if mode.is_commit() {
            EntryKind::Submodule
        } else {
            EntryKind::File
        };

        // Only files carry a size here; measuring a directory would mean walking it per row.
        let size = match kind {
            EntryKind::File => {
                let id = repo
                    .find_object(entry.oid().to_owned())
                    .map_err(GitError::from_git)?;
                Some(id.into_blob().data.len() as u64)
            }
            _ => None,
        };

        listing.push(TreeEntry {
            kind,
            name,
            path: entry_path,
            size,
        });
    }

    Ok(TreeListing {
        path: resolved,
        entries: listing,
    })
}

/// A file at a revision, as bytes. The name is returned alongside because a tree entry carries it.
pub fn blob_bytes(repo: &gix::Repository, revision: &str, path: &str) -> Result<(String, Vec<u8>)> {
    let commit = peeled_commit(repo, revision)?;
    let mut tree = commit.tree().map_err(GitError::from_git)?;

    let entry = tree
        .peel_to_entry_by_path(path)
        .map_err(GitError::from_git)?
        .ok_or_else(|| GitError::MissingPath {
            path: path.to_owned(),
            revision: revision.to_owned(),
        })?;

    if entry.mode().is_tree() {
        return Err(GitError::MissingPath {
            path: path.to_owned(),
            revision: revision.to_owned(),
        });
    }

    let name = entry.filename().to_string();
    let object = entry.object().map_err(GitError::from_git)?;
    // The blob is a borrowed view with a destructor, so its bytes are copied out rather than moved.
    Ok((name, object.into_blob().data.clone()))
}

/// A file at a revision, with its contents.
pub fn blob(repo: &gix::Repository, revision: &str, path: &str) -> Result<Blob> {
    let (name, data) = blob_bytes(repo, revision, path)?;
    let sha = blob_id(repo, revision, path)?;

    let binary = is_binary(&data);
    let text = (!binary && data.len() <= MAX_TEXT_BYTES)
        .then(|| String::from_utf8_lossy(&data).into_owned());

    Ok(Blob {
        name,
        path: path.to_owned(),
        size: data.len() as u64,
        sha,
        binary,
        text,
        content: crate::diff::base64(&data),
    })
}

/// The object id of a file at a revision.
fn blob_id(repo: &gix::Repository, revision: &str, path: &str) -> Result<Sha> {
    let commit = peeled_commit(repo, revision)?;
    let mut tree = commit.tree().map_err(GitError::from_git)?;

    let entry = tree
        .peel_to_entry_by_path(path)
        .map_err(GitError::from_git)?
        .ok_or_else(|| GitError::MissingPath {
            path: path.to_owned(),
            revision: revision.to_owned(),
        })?;

    Sha::parse(&entry.id().detach().to_string()).map_err(GitError::from)
}

/// The repository's readme at a revision, and the path it was found at.
pub fn readme(repo: &gix::Repository, revision: &str) -> Result<Option<(String, String)>> {
    let commit = match peeled_commit(repo, revision) {
        Ok(commit) => commit,
        // An empty repository has no readme, which is not an error worth reporting to a visitor.
        Err(GitError::UnknownRevision { .. }) => return Ok(None),
        Err(other) => return Err(other),
    };

    let mut tree = commit.tree().map_err(GitError::from_git)?;
    for candidate in README_CANDIDATES {
        let Some(entry) = tree
            .peel_to_entry_by_path(candidate)
            .map_err(GitError::from_git)?
        else {
            continue;
        };
        if entry.mode().is_tree() {
            continue;
        }

        let object = entry.object().map_err(GitError::from_git)?;
        let data = object.into_blob().data.clone();
        if is_binary(&data) {
            return Ok(None);
        }
        return Ok(Some((
            (*candidate).to_owned(),
            String::from_utf8_lossy(&data).into_owned(),
        )));
    }

    Ok(None)
}

/// Build a commit summary from a gix commit.
pub fn summary(commit: &gix::Commit<'_>) -> Result<CommitSummary> {
    let message = commit.message().map_err(GitError::from_git)?;
    let author = signature(commit.author().map_err(GitError::from_git)?)?;
    let committer = signature(commit.committer().map_err(GitError::from_git)?)?;

    let parents = commit
        .parent_ids()
        .map(|id| Sha::parse(&id.detach().to_string()).map_err(GitError::from))
        .collect::<Result<Vec<_>>>()?;

    Ok(CommitSummary {
        sha: Sha::parse(&commit.id().detach().to_string())?,
        message: message.title.to_string(),
        when: author.when,
        author,
        committer,
        parents,
    })
}

/// Assemble a commit detail once the change set is known.
pub fn detail(
    summary: CommitSummary,
    body: String,
    files: Vec<scabby_core::git::FileChange>,
) -> CommitDetail {
    let stats = DiffStats {
        additions: files.iter().map(|file| file.additions).sum(),
        deletions: files.iter().map(|file| file.deletions).sum(),
        files: files.len() as u64,
    };
    CommitDetail {
        summary,
        body,
        files,
        stats,
    }
}

fn peeled_commit<'repo>(
    repo: &'repo gix::Repository,
    revision: &str,
) -> Result<gix::Commit<'repo>> {
    let id = repo
        .rev_parse_single(revision)
        .map_err(|_| GitError::UnknownRevision {
            revision: revision.to_owned(),
        })?;

    let object = id.object().map_err(GitError::from_git)?;
    object
        .try_into_commit()
        .map_err(|_| GitError::UnknownRevision {
            revision: revision.to_owned(),
        })
}

fn signature(reference: gix::actor::SignatureRef<'_>) -> Result<Signature> {
    // A signature date is leniently parsed by gix, so a malformed one is reported rather than
    // silently replaced with the epoch: a wrong commit date is worse than a failed page load.
    let time = reference.time().map_err(GitError::from_git)?;
    let offset = UtcOffset::from_whole_seconds(time.offset).unwrap_or(UtcOffset::UTC);
    let when = OffsetDateTime::from_unix_timestamp(time.seconds)
        .map_err(GitError::from_git)?
        .to_offset(offset)
        .into();

    Ok(Signature {
        // A commit header is bytes, not necessarily UTF-8, so it is converted the lossy way rather
        // than refused: a name in a legacy encoding should still render.
        name: String::from_utf8_lossy(reference.name).trim().to_owned(),
        email: String::from_utf8_lossy(reference.email).trim().to_owned(),
        when,
    })
}

/// Whether a blob looks like something a browser should not be handed as text.
///
/// The same rule git uses: a NUL byte in the first few kilobytes.
fn is_binary(data: &[u8]) -> bool {
    data.iter().take(8000).any(|byte| *byte == 0)
}

/// Re-exported so callers do not have to reach into `diff` for a timestamp conversion.
pub fn timestamp(time: gix::date::Time) -> Timestamp {
    let offset = UtcOffset::from_whole_seconds(time.offset).unwrap_or(UtcOffset::UTC);
    OffsetDateTime::from_unix_timestamp(time.seconds)
        .unwrap_or(OffsetDateTime::UNIX_EPOCH)
        .to_offset(offset)
        .into()
}
