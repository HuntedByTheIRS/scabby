//! Commands that act on the instance rather than through it.

use anyhow::{Context as _, bail};
use scabby_core::config::Config;
use scabby_core::crypto::{MIN_PASSWORD_LEN, hash_password, hex, random_bytes};
use scabby_core::names::Name;
use scabby_db::{Db, users};
use scabby_git::GitStore;

use crate::cli::AdminCommand;

/// Run an administrative command.
pub async fn run(config: &Config, command: AdminCommand) -> anyhow::Result<()> {
    match command {
        AdminCommand::CreateUser {
            username,
            email,
            admin,
            password_stdin,
        } => create_user(config, &username, &email, admin, password_stdin).await,
        AdminCommand::InstallHooks => install_hooks(config).await,
    }
}

/// Create an account, which is the only way onto an instance with registration closed.
async fn create_user(
    config: &Config,
    username: &str,
    email: &str,
    admin: bool,
    password_stdin: bool,
) -> anyhow::Result<()> {
    let name = Name::parse(username).context("invalid username")?;

    let email = email.trim().to_owned();
    if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        bail!("email must look like an address, got {email}");
    }

    // A generated password is printed once and never stored in the clear; one read from stdin never
    // appears in this process's output at all.
    let generated = !password_stdin;
    let password = if password_stdin {
        let mut line = String::new();
        std::io::BufRead::read_line(&mut std::io::stdin().lock(), &mut line)
            .context("reading a password from stdin")?;
        line.trim_end_matches(['\n', '\r']).to_owned()
    } else {
        hex(&random_bytes(18))
    };

    if password.chars().count() < MIN_PASSWORD_LEN {
        bail!("password must be at least {MIN_PASSWORD_LEN} characters");
    }

    let hash = hash_password(&password).context("hashing the password")?;

    let db = Db::connect(config.database_url(), config.max_connections)
        .await
        .context("connecting to PostgreSQL")?;
    let user = users::create(&db, &name, &email, &hash, admin)
        .await
        .context("creating the account")?;

    if generated {
        // Deliberately stdout rather than a log line: this is the answer to the command, and it is
        // shown exactly once.
        println!("password for {}: {password}", user.username);
    }

    println!(
        "created {} <{}>, administrator: {}",
        user.username, email, user.is_admin
    );
    db.close().await;
    Ok(())
}

/// Rewrite every repository's hooks to call the binary that is running now.
async fn install_hooks(config: &Config) -> anyhow::Result<()> {
    let binary = std::env::current_exe().context("finding the running executable")?;
    let store = GitStore::new(&config.data_dir).with_hook_command(&binary);

    let slugs = store.list().context("listing repositories")?;
    let mut written = 0;
    for slug in &slugs {
        match store.install_hooks(slug) {
            Ok(()) => written += 1,
            // One unreadable repository should not stop the pass: the command is idempotent, so
            // fixing the permission and running it again is the whole recovery.
            Err(error) => {
                tracing::warn!(repository = %slug.full_name(), %error, "could not write hooks")
            }
        }
    }

    println!("rewrote hooks in {written} of {} repositories", slugs.len());
    Ok(())
}
