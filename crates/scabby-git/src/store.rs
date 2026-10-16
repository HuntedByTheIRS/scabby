//! Where bare repositories live on disk.

use std::path::{Path, PathBuf};
use std::process::Command;

use rayon::prelude::*;
use scabby_core::git::GitStats;
use scabby_core::names::Slug;

use crate::error::GitError;
use crate::hooks;
use crate::{Result, read};

/// The directory tree of bare repositories, one directory per repository, `owner/name.git`.
#[derive(Debug, Clone)]
pub struct GitStore {
    root: PathBuf,
    hook_command: PathBuf,
    git: PathBuf,
}

impl GitStore {
    /// Wrap a directory. The directory is created when a repository is first made, not here.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            hook_command: PathBuf::from(hooks::FALLBACK_COMMAND),
            git: PathBuf::from("git"),
        }
    }

    /// Use a specific git binary for the operations that run git as a subprocess.
    #[must_use]
    pub fn with_git(mut self, git: impl Into<PathBuf>) -> Self {
        self.git = git.into();
        self
    }

    /// Record the binary the hooks should call back into, usually [`std::env::current_exe`].
    #[must_use]
    pub fn with_hook_command(mut self, command: impl Into<PathBuf>) -> Self {
        self.hook_command = command.into();
        self
    }

    /// Root directory of the store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Directory of one repository.
    pub fn path(&self, slug: &Slug) -> PathBuf {
        self.root.join(slug.relative_path())
    }

    /// Whether the repository exists on disk.
    pub fn exists(&self, slug: &Slug) -> bool {
        self.path(slug).join("HEAD").is_file()
    }

    /// Create a bare repository and install its hooks.
    ///
    /// `git init` runs as a subprocess even though gix can initialise a repository, because the
    /// init is also what a deployment inspector checks with `git rev-parse` when something is wrong.
    pub fn create(&self, slug: &Slug, default_branch: &str) -> Result<()> {
        let dir = self.path(slug);
        if self.exists(slug) {
            return Ok(());
        }

        if let Some(parent) = dir.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let output = Command::new(&self.git)
            .arg("init")
            .arg("--bare")
            .arg("--quiet")
            .arg(format!("--initial-branch={default_branch}"))
            .arg(&dir)
            .output()?;

        if !output.status.success() {
            return Err(GitError::Command {
                command: "init".to_owned(),
                status: output.status.code().unwrap_or(-1),
                message: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            });
        }

        hooks::install(&dir, slug, &self.hook_command)?;
        Ok(())
    }

    /// Rewrite the hooks, which is what an upgrade does when the callback command changes.
    pub fn install_hooks(&self, slug: &Slug) -> Result<()> {
        hooks::install(&self.path(slug), slug, &self.hook_command)?;
        Ok(())
    }

    /// Delete a repository directory.
    pub fn remove(&self, slug: &Slug) -> Result<()> {
        let dir = self.path(slug);
        if !dir.exists() {
            return Err(GitError::MissingRepository {
                slug: slug.full_name(),
            });
        }
        std::fs::remove_dir_all(dir)?;
        Ok(())
    }

    /// Open a repository for reading.
    pub fn open(&self, slug: &Slug) -> Result<gix::Repository> {
        let dir = self.path(slug);
        if !self.exists(slug) {
            return Err(GitError::MissingRepository {
                slug: slug.full_name(),
            });
        }
        // A bare repository is opened as-is; the on-disk format is v0 unless someone opts in.
        gix::open(&dir).map_err(GitError::from_git)
    }

    /// Bytes the repository occupies, as the filesystem counts them.
    pub fn disk_bytes(&self, slug: &Slug) -> Result<u64> {
        Ok(disk_usage(&self.path(slug))?)
    }

    /// Everything the API reports about a repository's git side.
    pub fn stats(&self, slug: &Slug, default_branch: &str) -> Result<GitStats> {
        let repo = self.open(slug)?;
        let branches = read::branches(&repo, default_branch)?;
        let tags = read::tags(&repo)?;
        let head = read::head_commit(&repo, default_branch)?;
        let commit_count = read::commit_count(&repo, default_branch, read::COMMIT_COUNT_LIMIT)?;

        Ok(GitStats {
            branches: branches.len(),
            tags: tags.len(),
            disk_bytes: self.disk_bytes(slug)?,
            commit_count,
            head_commit: head,
        })
    }

    /// Stats for many repositories at once, each with its own default branch.
    ///
    /// Each repository is opened, walked, and measured independently, which is what rayon is for:
    /// the work is CPU-bound, there is no shared mutable state, and a page of repository cards wants
    /// all of it in one request. Results come back in the order asked for, each one a `Result`, so a
    /// single broken directory leaves the rest of the page rendered.
    pub fn stats_all(&self, targets: &[(Slug, String)]) -> Vec<Result<GitStats>> {
        targets
            .par_iter()
            .map(|(slug, default_branch)| self.stats(slug, default_branch))
            .collect()
    }

    /// Every repository under the root, for a reconcile pass.
    pub fn list(&self) -> Result<Vec<Slug>> {
        let mut found = Vec::new();
        if !self.root.exists() {
            return Ok(found);
        }

        for owner in std::fs::read_dir(&self.root)? {
            let owner = owner?;
            if !owner.file_type()?.is_dir() {
                continue;
            }
            let owner_name = owner.file_name().to_string_lossy().into_owned();

            for repo in std::fs::read_dir(owner.path())? {
                let repo = repo?;
                let name = repo.file_name().to_string_lossy().into_owned();
                let Some(name) = name.strip_suffix(".git") else {
                    continue;
                };
                // A directory that is not a valid slug is not a repository Scabby created, and
                // skipping it keeps one stray file from failing the whole listing.
                if let Ok(slug) = Slug::parse(&format!("{owner_name}/{name}")) {
                    found.push(slug);
                }
            }
        }

        // Sorted by the string a user sees, because `Slug` is deliberately not `Ord`: names are
        // compared folded for identity and rendered as typed for display, and those are two orders.
        found.sort_by_key(Slug::full_name);
        Ok(found)
    }
}

/// Sum of file sizes under a directory, following no symlinks.
fn disk_usage(path: &Path) -> std::io::Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let mut total = 0_u64;
    for entry in walkdir::WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .flatten()
    {
        if entry.file_type().is_file() {
            total = total.saturating_add(entry.metadata().map(|meta| meta.len()).unwrap_or(0));
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_layout_is_owner_then_name_dot_git() {
        let store = GitStore::new("/srv/scabby/repos");
        let slug = Slug::parse("ada/scabby").unwrap();
        assert_eq!(
            store.path(&slug),
            PathBuf::from("/srv/scabby/repos/ada/scabby.git")
        );
    }
}
