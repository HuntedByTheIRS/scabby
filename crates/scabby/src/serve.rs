//! Running the server.

use std::sync::Arc;

use anyhow::Context as _;
use scabby_core::config::Config;
use scabby_db::Db;
use scabby_git::{GitStore, SmartHttp};
use scabby_web::AppState;

use crate::cli::Settings;

/// Apply migrations and exit.
pub async fn migrate(config: &Config) -> anyhow::Result<()> {
    let db = connect(config).await?;
    db.migrate().await.context("applying migrations")?;
    tracing::info!("migrations are up to date");
    db.close().await;
    Ok(())
}

/// Serve the API, the user interface, and git.
pub async fn run(config: &Config, settings: &Settings) -> anyhow::Result<()> {
    let db = connect(config).await?;

    if config.migrate_on_start {
        db.migrate().await.context("applying migrations")?;
    }

    std::fs::create_dir_all(&config.data_dir)
        .with_context(|| format!("creating {}", config.data_dir.display()))?;

    // Hooks call back into this exact file, so a checkout that was never installed still works and
    // a binary that moved needs `scabby admin install-hooks` rather than a silent no-op.
    let binary = std::env::current_exe().context("finding the running executable")?;
    let store = GitStore::new(&config.data_dir)
        .with_git(settings.git.clone())
        .with_hook_command(&binary);

    // The git children start the hooks, and the hooks call back into this binary: passing the
    // resolved settings down through the environment is what lets a hook find the same database the
    // server is using, even when that came from a config file rather than from the environment.
    let transport = SmartHttp::new()
        .with_git(settings.git.clone())
        .with_env("SCABBY_DATABASE_URL", config.database_url())
        .with_env("SCABBY_DATA_DIR", config.data_dir.display().to_string());

    let state =
        AppState::new(Arc::new(config.clone()), db.clone(), store).with_transport(transport);
    let app = scabby_web::router(state);

    let listener = tokio::net::TcpListener::bind(config.listen)
        .await
        .with_context(|| format!("binding {}", config.listen))?;
    let address = listener.local_addr().context("reading the bound address")?;

    tracing::info!(
        address = %address,
        base_url = %config.origin(),
        data_dir = %config.data_dir.display(),
        "scabby is listening"
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await
        .context("serving requests")?;

    db.close().await;
    Ok(())
}

/// Open the pool, naming the database host but never the password.
async fn connect(config: &Config) -> anyhow::Result<Db> {
    // The URL is deliberately not in the message: sqlx's own errors do not carry it, and this one
    // is the line an operator reads in a log they may paste into an issue.
    let db = Db::connect(config.database_url(), config.max_connections)
        .await
        .context("connecting to PostgreSQL")?;
    Ok(db)
}

/// Wait for a signal that means stop, and let the current requests finish.
async fn shutdown() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => {
                tracing::warn!(%error, "could not listen for a termination signal");
                std::future::pending::<()>().await;
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }

    tracing::info!("shutting down");
}
