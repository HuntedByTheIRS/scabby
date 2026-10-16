//! The change set of a commit, read from `git diff-tree`.
//!
//! gix can diff two trees, and its line counter is young enough that its numbers do not always match
//! what `git diff --stat` shows. A code host whose additions and deletions disagree with the
//! developer's terminal is a code host nobody trusts, so the counts come from git itself.
//!
//! Two passes over the same commit: `--name-status` for what happened to each file and `--numstat`
//! for the line counts. Both run with `-z`, so paths with newlines or unicode arrive intact and
//! unquoted.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use scabby_core::git::{ChangeStatus, FileChange};

use crate::Result;
use crate::error::GitError;

/// Base64 for blob contents, the one place the encoding is chosen.
pub fn base64(data: &[u8]) -> String {
    STANDARD.encode(data)
}

/// Files a commit touched, always measured against the first parent.
///
/// `-M` turns on rename detection, so a moved file is reported as a rename rather than as a delete
/// plus an add, which is what a reviewer expects to see.
pub fn file_changes(repo_dir: &Path, sha: &str, parent: Option<&str>) -> Result<Vec<FileChange>> {
    let statuses = run(repo_dir, "--name-status", sha, parent)?;
    let counts = run(repo_dir, "--numstat", sha, parent)?;

    let statuses = parse_name_status(&statuses);
    let counts = parse_numstat(&counts);

    let mut changes = Vec::with_capacity(statuses.len());
    for (status, path) in statuses {
        let (additions, deletions, binary) = counts.get(&path).copied().unwrap_or((0, 0, false));

        changes.push(FileChange {
            path,
            status,
            additions,
            deletions,
            binary,
        });
    }
    Ok(changes)
}

/// Run one `git diff-tree` pass and return its raw NUL-separated output.
fn run(repo_dir: &Path, format: &str, sha: &str, parent: Option<&str>) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    command
        .current_dir(repo_dir)
        .arg("diff-tree")
        .arg("-r")
        .arg("-M")
        .arg("-z")
        .arg(format);

    match parent {
        // Against an explicit parent, so a merge commit reports the change a reviewer sees on the
        // first-parent view rather than nothing at all.
        Some(parent) => {
            command.arg(parent).arg(sha);
        }
        None => {
            command.arg("--root").arg("--no-commit-id").arg(sha);
        }
    }

    let output = command.output()?;
    if !output.status.success() {
        return Err(GitError::Command {
            command: format!("diff-tree {format}"),
            status: output.status.code().unwrap_or(-1),
            message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    Ok(output.stdout)
}

/// Parse `--name-status -z`: `S\0path\0`, and `R<score>\0old\0new\0` for a rename.
fn parse_name_status(bytes: &[u8]) -> Vec<(ChangeStatus, String)> {
    let fields: Vec<&[u8]> = bytes
        .split(|byte| *byte == 0)
        .filter(|f| !f.is_empty())
        .collect();
    let mut changes = Vec::new();
    let mut index = 0;

    while index < fields.len() {
        let code = String::from_utf8_lossy(fields[index]).into_owned();
        let letter = code.chars().next().unwrap_or('M');

        match letter {
            'R' | 'C' => {
                // Renames and copies carry the old path and then the new one; the new path is what
                // the UI links to.
                let Some(new_path) = fields.get(index + 2) else {
                    break;
                };
                let status = if letter == 'C' {
                    ChangeStatus::Added
                } else {
                    ChangeStatus::Renamed
                };
                changes.push((status, String::from_utf8_lossy(new_path).into_owned()));
                index += 3;
            }
            _ => {
                let Some(path) = fields.get(index + 1) else {
                    break;
                };
                let status = match letter {
                    'A' => ChangeStatus::Added,
                    'D' => ChangeStatus::Deleted,
                    _ => ChangeStatus::Modified,
                };
                changes.push((status, String::from_utf8_lossy(path).into_owned()));
                index += 2;
            }
        }
    }

    changes
}

/// Parse `--numstat -z`: `add\tdel\tpath\0`, or `add\tdel\t\0old\0new\0` for a rename.
///
/// A binary file reports `-` for both counts, which is how it is recognised.
fn parse_numstat(bytes: &[u8]) -> HashMap<String, (u64, u64, bool)> {
    let fields: Vec<&[u8]> = bytes
        .split(|byte| *byte == 0)
        .filter(|f| !f.is_empty())
        .collect();
    let mut counts = HashMap::new();
    let mut index = 0;

    while index < fields.len() {
        let header = String::from_utf8_lossy(fields[index]).into_owned();
        let mut parts = header.splitn(3, '\t');
        let additions = parts.next().unwrap_or("-");
        let deletions = parts.next().unwrap_or("-");
        let path = parts.next().unwrap_or("");

        let binary = additions == "-" && deletions == "-";
        let additions = additions.parse().unwrap_or(0);
        let deletions = deletions.parse().unwrap_or(0);

        if path.is_empty() {
            // A rename: the paths follow as their own fields.
            let (Some(_old), Some(new)) = (fields.get(index + 1), fields.get(index + 2)) else {
                break;
            };
            counts.insert(
                String::from_utf8_lossy(new).into_owned(),
                (additions, deletions, binary),
            );
            index += 3;
        } else {
            counts.insert(path.to_owned(), (additions, deletions, binary));
            index += 1;
        }
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_name_status_with_renames() {
        let raw = b"M\0src/main.rs\0A\0src/lib.rs\0R093\0old.rs\0new.rs\0D\0gone.rs\0";
        let parsed = parse_name_status(raw);

        assert_eq!(parsed.len(), 4);
        assert_eq!(
            parsed[0],
            (ChangeStatus::Modified, "src/main.rs".to_owned())
        );
        assert_eq!(parsed[1], (ChangeStatus::Added, "src/lib.rs".to_owned()));
        assert_eq!(parsed[2], (ChangeStatus::Renamed, "new.rs".to_owned()));
        assert_eq!(parsed[3], (ChangeStatus::Deleted, "gone.rs".to_owned()));
    }

    #[test]
    fn parses_numstat_with_binary_and_rename() {
        let raw = b"1\t0\tsrc/main.rs\0-\t-\tlogo.png\0";
        let raw = [raw.as_slice(), b"2\t3\t\0old.rs\0new.rs\0".as_slice()].concat();
        let parsed = parse_numstat(&raw);

        assert_eq!(parsed.get("src/main.rs"), Some(&(1, 0, false)));
        assert_eq!(parsed.get("logo.png"), Some(&(0, 0, true)));
        assert_eq!(parsed.get("new.rs"), Some(&(2, 3, false)));
    }
}
