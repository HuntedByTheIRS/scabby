//! Storage tests against a real Postgres.
//!
//! Set `SCABBY_TEST_DATABASE_URL` to run them; without it each test reports that it was skipped, so
//! `cargo test` stays green on a machine with no database while CI gets the coverage. Every test
//! uses names containing the current time, so a leftover row from an earlier run cannot turn into a
//! false failure.

use std::time::{SystemTime, UNIX_EPOCH};

use scabby_core::Error;
use scabby_core::event::EventKind;
use scabby_core::issue::{IssueFilter, IssueState};
use scabby_core::names::{Name, Slug};
use scabby_core::pagination::Page;
use scabby_core::repo::Visibility;
use scabby_core::user::User;
use scabby_db::{Db, events, issues, repos, tokens, users};

async fn database() -> Option<Db> {
    let url = std::env::var("SCABBY_TEST_DATABASE_URL").ok()?;
    let db = Db::connect(&url, 4)
        .await
        .expect("connect to the test database");
    db.migrate().await.expect("run migrations");
    Some(db)
}

fn suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    format!("{}{}", std::process::id(), nanos % 1_000_000)
}

async fn make_user(db: &Db, prefix: &str) -> User {
    let username = Name::parse(&format!("{prefix}{}", suffix())).expect("valid username");
    users::create(
        db,
        &username,
        &format!("{username}@example.test"),
        "hash",
        false,
    )
    .await
    .expect("create user")
}

async fn make_repo(db: &Db, owner: &User, visibility: Visibility) -> scabby_core::repo::Repository {
    let name = Name::parse(&format!("repo{}", suffix())).expect("valid repo name");
    repos::create(
        db,
        owner.id.get(),
        &name,
        Some("test repository"),
        visibility,
        "main",
    )
    .await
    .expect("create repository")
}

#[tokio::test]
async fn users_round_trip() {
    let Some(db) = database().await else {
        eprintln!("skipped: SCABBY_TEST_DATABASE_URL is not set");
        return;
    };

    let user = make_user(&db, "roundtrip").await;
    let fetched = users::by_username(&db, &user.username.as_str().to_uppercase(), true)
        .await
        .expect("lookup is case-insensitive");
    assert_eq!(fetched.id, user.id);
    assert_eq!(
        fetched.email.as_deref(),
        Some(format!("{}@example.test", user.username).as_str())
    );

    let updated = users::update_profile(
        &db,
        user.id.get(),
        Some("Ada Lovelace"),
        Some("first programmer"),
    )
    .await
    .expect("update profile");
    assert_eq!(updated.display_name.as_deref(), Some("Ada Lovelace"));
    assert_eq!(updated.bio.as_deref(), Some("first programmer"));
}

#[tokio::test]
async fn duplicate_username_is_a_conflict() {
    let Some(db) = database().await else {
        eprintln!("skipped: SCABBY_TEST_DATABASE_URL is not set");
        return;
    };

    let user = make_user(&db, "dupe").await;
    let error = users::create(
        &db,
        &Name::parse(user.username.as_str().to_uppercase().as_str()).unwrap(),
        "other@example.test",
        "hash",
        false,
    )
    .await
    .expect_err("a case variant of the same handle must collide");

    match Error::from(error) {
        Error::Conflict { resource, .. } => assert_eq!(resource, "user"),
        other => panic!("expected a conflict, got {other:?}"),
    }
}

#[tokio::test]
async fn private_repositories_stay_out_of_anonymous_lists() {
    let Some(db) = database().await else {
        eprintln!("skipped: SCABBY_TEST_DATABASE_URL is not set");
        return;
    };

    let owner = make_user(&db, "visibility").await;
    let public = make_repo(&db, &owner, Visibility::Public).await;
    let private = make_repo(&db, &owner, Visibility::Private).await;

    let (anonymous, _) = repos::list_visible(&db, None, Page::new(None, Some(100)))
        .await
        .unwrap();
    assert!(anonymous.iter().any(|repo| repo.id == public.id));
    assert!(!anonymous.iter().any(|repo| repo.id == private.id));

    let (own, _) = repos::list_visible(&db, Some(owner.id.get()), Page::new(None, Some(100)))
        .await
        .unwrap();
    assert!(own.iter().any(|repo| repo.id == private.id));

    let (owned, _) = repos::list_owned(&db, owner.id.get(), None, Page::default())
        .await
        .unwrap();
    assert_eq!(
        owned.len(),
        1,
        "anonymous view of the owner's page shows the public repo only"
    );
}

#[tokio::test]
async fn repository_lookup_and_update() {
    let Some(db) = database().await else {
        eprintln!("skipped: SCABBY_TEST_DATABASE_URL is not set");
        return;
    };

    let owner = make_user(&db, "lookup").await;
    let repo = make_repo(&db, &owner, Visibility::Public).await;
    let slug = Slug::new(owner.username.clone(), repo.name.clone());

    let found = repos::by_slug(&db, &slug).await.expect("by slug");
    assert_eq!(found.id, repo.id);
    assert_eq!(found.open_issue_count, 0);

    let patch = repos::RepositoryPatch {
        description: Some(String::new()),
        visibility: Some(Visibility::Private),
        default_branch: Some("trunk".to_owned()),
    };
    let updated = repos::update(&db, repo.id.get(), &patch)
        .await
        .expect("update");
    assert_eq!(
        updated.description, None,
        "an empty description clears the column"
    );
    assert!(updated.visibility.is_private());
    assert_eq!(updated.default_branch, "trunk");

    repos::set_size(&db, repo.id.get(), 4096)
        .await
        .expect("set size");
    assert_eq!(
        repos::by_id(&db, repo.id.get()).await.unwrap().size_bytes,
        4096
    );

    repos::delete(&db, repo.id.get()).await.expect("delete");
    assert!(repos::by_slug(&db, &slug).await.is_err());
}

#[tokio::test]
async fn issue_numbers_are_per_repository_and_increment() {
    let Some(db) = database().await else {
        eprintln!("skipped: SCABBY_TEST_DATABASE_URL is not set");
        return;
    };

    let owner = make_user(&db, "issues").await;
    let other = make_user(&db, "issues").await;
    let repo = make_repo(&db, &owner, Visibility::Public).await;
    let other_repo = make_repo(&db, &other, Visibility::Public).await;

    let first = issues::create(&db, repo.id.get(), owner.id.get(), "First", Some("body")).await;
    let first = first.expect("first issue");
    let second = issues::create(&db, repo.id.get(), owner.id.get(), "Second", None)
        .await
        .expect("second issue");
    assert_eq!(first.number, 1);
    assert_eq!(second.number, 2);

    let elsewhere = issues::create(&db, other_repo.id.get(), other.id.get(), "First", None)
        .await
        .expect("issue in another repository");
    assert_eq!(elsewhere.number, 1, "numbering restarts per repository");

    let comment =
        issues::add_comment(&db, repo.id.get(), first.number, owner.id.get(), "a reply").await;
    let comment = comment.expect("comment");
    assert_eq!(comment.issue_number, 1);

    let reloaded = issues::by_number(&db, repo.id.get(), first.number)
        .await
        .unwrap();
    assert_eq!(reloaded.comment_count, 1);

    let (comments, total) = issues::comments(&db, repo.id.get(), first.number, Page::default())
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(comments[0].body, "a reply");

    let closed = issues::update(
        &db,
        repo.id.get(),
        first.number,
        &issues::IssuePatch {
            state: Some(IssueState::Closed),
            ..Default::default()
        },
    )
    .await
    .expect("close");
    assert_eq!(closed.state, IssueState::Closed);
    assert!(closed.closed_at.is_some());

    let (open, open_total) = issues::list(&db, repo.id.get(), IssueFilter::Open, Page::default())
        .await
        .unwrap();
    assert_eq!(open_total, 1);
    assert_eq!(open[0].number, 2);

    let (all, all_total) = issues::list(&db, repo.id.get(), IssueFilter::All, Page::default())
        .await
        .unwrap();
    assert_eq!(all_total, 2);
    assert_eq!(all.len(), 2);

    let open_count = repos::by_id(&db, repo.id.get())
        .await
        .unwrap()
        .open_issue_count;
    assert_eq!(open_count, 1);
}

#[tokio::test]
async fn tokens_authenticate_and_expire() {
    let Some(db) = database().await else {
        eprintln!("skipped: SCABBY_TEST_DATABASE_URL is not set");
        return;
    };

    let user = make_user(&db, "tokens").await;
    let hash = vec![7_u8; 32];

    let token = tokens::create_token(
        &db,
        user.id.get(),
        "ci",
        "scabby_pat_abc123",
        &hash,
        &["repo".to_owned()],
        None,
    )
    .await
    .expect("create token");
    assert_eq!(token.name, "ci");
    assert_eq!(
        tokens::tokens_for_user(&db, user.id.get())
            .await
            .unwrap()
            .len(),
        1
    );

    let authenticated = tokens::token_user(&db, &hash)
        .await
        .unwrap()
        .expect("token resolves");
    assert_eq!(authenticated.id, user.id);
    assert!(
        tokens::token_user(&db, &[9_u8; 32])
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        tokens::tokens_for_user(&db, user.id.get()).await.unwrap()[0]
            .last_used_at
            .is_some(),
        "using a token records the use"
    );

    let expired = tokens::create_token(
        &db,
        user.id.get(),
        "old",
        "scabby_pat_def456",
        &[8_u8; 32],
        &[],
        Some(time::OffsetDateTime::now_utc() - time::Duration::hours(1)),
    )
    .await
    .expect("create expired token");
    assert!(
        tokens::token_user(&db, &[8_u8; 32])
            .await
            .unwrap()
            .is_none()
    );

    tokens::delete_token(&db, user.id.get(), token.id.get())
        .await
        .expect("delete");
    tokens::delete_token(&db, user.id.get(), expired.id.get())
        .await
        .expect("delete expired");
    assert!(
        tokens::tokens_for_user(&db, user.id.get())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        tokens::delete_token(&db, user.id.get(), token.id.get())
            .await
            .is_err()
    );
}

#[tokio::test]
async fn sessions_and_ssh_keys() {
    let Some(db) = database().await else {
        eprintln!("skipped: SCABBY_TEST_DATABASE_URL is not set");
        return;
    };

    let user = make_user(&db, "sessions").await;
    let hash = vec![3_u8; 32];
    tokens::create_session(
        &db,
        user.id.get(),
        &hash,
        Some("test-agent"),
        time::OffsetDateTime::now_utc() + time::Duration::days(7),
    )
    .await
    .expect("create session");

    let resolved = tokens::session_user(&db, &hash)
        .await
        .unwrap()
        .expect("session resolves");
    assert_eq!(resolved.id, user.id);
    assert!(
        tokens::session_user(&db, &[4_u8; 32])
            .await
            .unwrap()
            .is_none()
    );

    tokens::delete_session(&db, &hash)
        .await
        .expect("delete session");
    assert!(tokens::session_user(&db, &hash).await.unwrap().is_none());

    let key = tokens::create_ssh_key(
        &db,
        user.id.get(),
        "laptop",
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample ada@laptop",
        "SHA256:examplefingerprint",
    )
    .await
    .expect("create key");
    assert_eq!(
        tokens::ssh_keys_for_user(&db, user.id.get())
            .await
            .unwrap()
            .len(),
        1
    );

    let duplicate = tokens::create_ssh_key(
        &db,
        user.id.get(),
        "same key again",
        "ssh-ed25519 AAAA other",
        "SHA256:examplefingerprint",
    )
    .await
    .expect_err("the same fingerprint cannot be added twice");
    match Error::from(duplicate) {
        Error::Conflict { resource, .. } => assert_eq!(resource, "ssh key"),
        other => panic!("expected a conflict, got {other:?}"),
    }

    tokens::delete_ssh_key(&db, user.id.get(), key.id.get())
        .await
        .expect("delete key");
    assert!(
        tokens::ssh_keys_for_user(&db, user.id.get())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn activity_feed_reads_back_what_was_recorded() {
    let Some(db) = database().await else {
        eprintln!("skipped: SCABBY_TEST_DATABASE_URL is not set");
        return;
    };

    let owner = make_user(&db, "feed").await;
    let repo = make_repo(&db, &owner, Visibility::Public).await;

    events::record(
        &db,
        Some(owner.id.get()),
        Some(repo.id.get()),
        EventKind::RepositoryCreated,
    )
    .await
    .expect("record");

    let (feed, total) = events::recent(&db, Page::new(Some(1), Some(50)))
        .await
        .unwrap();
    assert!(total >= 1);
    let entry = feed
        .iter()
        .find(|event| {
            event
                .repository
                .as_ref()
                .is_some_and(|slug| slug.name == repo.name)
        })
        .expect("the recorded event is in the feed");
    assert_eq!(entry.kind, EventKind::RepositoryCreated);
    assert_eq!(entry.actor.as_ref().map(|actor| actor.id), Some(owner.id));
}

#[tokio::test]
async fn prune_drops_old_entries() {
    let Some(db) = database().await else {
        eprintln!("skipped: SCABBY_TEST_DATABASE_URL is not set");
        return;
    };

    let owner = make_user(&db, "prune").await;
    events::record(&db, Some(owner.id.get()), None, EventKind::UserRegistered)
        .await
        .unwrap();
    let removed = events::prune(
        &db,
        time::OffsetDateTime::now_utc() + time::Duration::days(1),
    )
    .await
    .unwrap();
    assert!(removed >= 1, "future cutoff removes everything recorded");
}
