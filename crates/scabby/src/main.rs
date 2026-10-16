//! The Scabby binary.
//!
//! One executable serves everything: the REST API, the user interface embedded in it, and the git
//! endpoints a clone or a push talks to. It also carries the commands that have to exist outside the
//! HTTP surface, which are the two that cannot be done over the API: applying migrations, and
//! creating the first account on an instance whose registration is closed.

mod admin;
mod cli;
mod hook;
mod resolve;
mod serve;

use std::process::ExitCode;

use clap::Parser as _;
use tracing_subscriber::EnvFilter;

use crate::cli::{Cli, Command};

fn main() -> ExitCode {
    let cli = Cli::parse();
    init_logging(&cli.settings.log_filter());

    // The runtime is built by hand rather than with an attribute, so a failure to start one is a
    // message rather than a panic, and so the exit code is a value this function returns.
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("scabby: could not start the async runtime: {error}");
            return ExitCode::FAILURE;
        }
    };

    match runtime.block_on(run(cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            // The chain, not just the last link: a database failure prints what sqlx said, and the
            // line above it says what Scabby was doing at the time.
            tracing::error!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

/// Dispatch, with the configuration resolved once for whichever command runs.
async fn run(cli: Cli) -> anyhow::Result<()> {
    let config = resolve::config(&cli.settings)?;

    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => serve::run(&config, &cli.settings).await,
        Command::Migrate => serve::migrate(&config).await,
        Command::Admin(command) => admin::run(&config, command).await,
        Command::Hook(command) => hook::run(&config, command).await,
    }
}

/// Install a subscriber, taking the filter from `--log` when it is usable.
///
/// A filter that does not parse is not fatal: the process still has work to do, and refusing to
/// start because a log directive has a typo would be a poor trade.
fn init_logging(filter: &str) {
    let filter = EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}
