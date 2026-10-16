//! Turning the command line, the environment, and the file into one configuration.
//!
//! Precedence, lowest first: built-in defaults, the TOML file, the environment, the flag. clap
//! cannot separate an environment value from a flag for the same argument, so those two share a
//! level and a flag wins where both are present, which is what someone typing a flag expects.

use scabby_core::Result;
use scabby_core::config::Config;

use crate::cli::Settings;

/// The configuration every command runs with.
pub fn config(settings: &Settings) -> Result<Config> {
    let mut config = Config::load(config_path(settings).as_deref())?;

    if let Some(listen) = settings.listen {
        config.listen = listen;
    }
    if let Some(url) = &settings.database_url {
        // The wrapper is what keeps the URL out of logs and error messages.
        config.database_url = url.as_str().to_owned().into();
    }
    if let Some(dir) = &settings.data_dir {
        config.data_dir = dir.clone();
    }
    if let Some(url) = &settings.base_url {
        config.base_url = url.clone();
    }
    if let Some(dir) = &settings.frontend_dir {
        config.frontend_dir = Some(dir.clone());
    }
    if let Some(mode) = settings.registration {
        config.registration = mode.into();
    }
    if let Some(max) = settings.max_connections {
        config.max_connections = max;
    }
    if let Some(migrate) = settings.migrate_on_start {
        config.migrate_on_start = migrate;
    }

    config.validate()?;
    Ok(config)
}

/// The file to read, which is the argument when it is given and `./scabby.toml` when that exists.
fn config_path(settings: &Settings) -> Option<std::path::PathBuf> {
    if settings.config.is_some() {
        return settings.config.clone();
    }

    let default = std::path::PathBuf::from("scabby.toml");
    default.exists().then_some(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::RegistrationMode;
    use std::net::SocketAddr;

    fn bare_settings() -> Settings {
        Settings {
            config: None,
            listen: None,
            database_url: None,
            data_dir: None,
            base_url: None,
            frontend_dir: None,
            registration: None,
            git: "git".into(),
            log: "info".to_owned(),
            max_connections: None,
            migrate_on_start: None,
        }
    }

    #[test]
    fn flags_beat_the_defaults() {
        let settings = Settings {
            listen: Some("0.0.0.0:9000".parse::<SocketAddr>().unwrap()),
            registration: Some(RegistrationMode::Closed),
            frontend_dir: Some("/srv/ui".into()),
            ..bare_settings()
        };

        let config = config(&settings).unwrap();
        assert_eq!(config.listen.port(), 9000);
        assert_eq!(
            config.registration,
            scabby_core::config::Registration::Closed
        );
        assert_eq!(config.frontend_dir, Some("/srv/ui".into()));
    }

    #[test]
    fn a_missing_config_file_named_on_the_command_line_is_an_error() {
        let settings = Settings {
            config: Some("/nonexistent/scabby.toml".into()),
            ..bare_settings()
        };
        assert!(config(&settings).is_err());
    }
}
