//! The callbacks git runs.
//!
//! These are not for people. A hook script installed in every bare repository runs
//! `scabby hook <name> --repo <owner/name>` and hands over its stdin, which is how git reports which
//! refs moved. The work here is deliberately small: record that a push happened, and refresh the
//! size the repository list shows. Anything that fails must not reject a push that already
//! completed, so the hook script swallows this program's exit status.

use anyhow::Context as _;
use scabby_core::config::Config;
use scabby_core::event::EventKind;
use scabby_core::names::Slug;
use scabby_db::{Db, events, repos, users};
use scabby_git::GitStore;

use crate::cli::{HookCommand, HookKind};

/// Run a hook.
pub async fn run(config: &Config, command: HookCommand) -> anyhow::Result<()> {
    match command.kind {
        HookKind::PostReceive { repo } => post_receive(config, &repo).await,
        HookKind::PreReceive { repo } => pre_receive(&repo),
    }
}

/// Record a push that git has already accepted.
async fn post_receive(config: &Config, repo: &str) -> anyhow::Result<()> {
    let moved = refs_moved(read_stdin()?);
    if moved.is_empty() {
        // A push that only deleted refs, or that moved nothing this hook cares about.
        return Ok(());
    }

    let slug = Slug::parse(repo).with_context(|| format!("reading the repository name {repo}"))?;
    let db = Db::connect(config.database_url(), config.max_connections)
        .await
        .context("connecting to PostgreSQL")?;
    let repository = repos::by_slug(&db, &slug)
        .await
        .context("looking up the repository")?;

    // The transport puts the authenticated username in REMOTE_USER, which git passes through to the
    // hook. It is attribution, not authorization: the push was authorized before git was started.
    let actor = match std::env::var("REMOTE_USER") {
        Ok(username) if !username.trim().is_empty() => {
            users::by_username(&db, username.trim(), false)
                .await
                .ok()
                .map(|user| user.id.get())
        }
        _ => None,
    };

    events::record(
        &db,
        actor,
        Some(repository.id.get()),
        EventKind::PushReceived,
    )
    .await
    .context("recording the push")?;

    // The API reports a stored size, and this is the one moment it can change.
    let store = GitStore::new(&config.data_dir);
    match store.disk_bytes(&slug) {
        Ok(bytes) => {
            if let Err(error) = repos::set_size(&db, repository.id.get(), bytes).await {
                tracing::warn!(repository = %slug.full_name(), %error, "could not store the size");
            }
        }
        Err(error) => tracing::warn!(repository = %slug.full_name(), %error, "could not measure"),
    }

    tracing::info!(
        repository = %slug.full_name(),
        refs = moved.len(),
        "push recorded"
    );
    db.close().await;
    Ok(())
}

/// `pre-receive` runs before the refs move, and nothing is policy-gated yet.
///
/// It exists so that adding a refusal later means changing this function rather than installing a
/// new hook in every repository that already exists.
fn pre_receive(repo: &str) -> anyhow::Result<()> {
    let moved = refs_moved(read_stdin()?);
    tracing::debug!(repo, refs = moved.len(), "pre-receive accepted every ref");
    Ok(())
}

/// Read everything git wrote to this process's stdin.
fn read_stdin() -> anyhow::Result<String> {
    std::io::read_to_string(std::io::stdin()).context("reading the ref list from stdin")
}

/// The refs a hook was told about.
///
/// git writes one line per ref: the old object id, the new object id, then the full ref name. An id
/// of all zeroes means the ref did not exist before, or does not exist after, so a line whose new id
/// is zeroes is a deletion and carries no commits.
///
/// The ids are checked to look like object ids before a line is believed. This input arrives from
/// another process, and three whitespace-separated words are not a ref update.
fn refs_moved(input: String) -> Vec<(String, String, String)> {
    let mut moved = Vec::new();

    for line in input.lines() {
        let mut fields = line.split_whitespace();
        let (Some(old), Some(new), Some(reference)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };

        if !is_object_id(old) || !is_object_id(new) {
            continue;
        }

        // Every digit zero: the ref is being deleted, so no commits arrived on this line.
        if new.bytes().all(|byte| byte == b'0') {
            continue;
        }

        moved.push((old.to_owned(), new.to_owned(), reference.to_owned()));
    }

    moved
}

/// Length of a SHA-1 object id in hex, the shortest id git writes into a hook.
const SHA1_HEX_LEN: usize = 40;

/// Whether a field looks like a git object id: hex, and as long as an abbreviated-to-full SHA-1.
fn is_object_id(field: &str) -> bool {
    field.len() >= SHA1_HEX_LEN && field.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletions_are_not_pushes() {
        let input = "\
0000000000000000000000000000000000000000 1111111111111111111111111111111111111111 refs/heads/main
1111111111111111111111111111111111111111 0000000000000000000000000000000000000000 refs/heads/gone
";
        let moved = refs_moved(input.to_owned());
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].2, "refs/heads/main");
    }

    #[test]
    fn junk_lines_are_skipped() {
        assert!(refs_moved("not a ref line\n\n".to_owned()).is_empty());
        assert!(
            refs_moved("1111 main refs/heads/main".to_owned()).is_empty(),
            "short ids are not ids"
        );
        assert!(
            refs_moved("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz refs/heads/main".to_owned())
                .is_empty(),
            "non-hex is not an id"
        );
        assert!(refs_moved(String::new()).is_empty());
    }
}
