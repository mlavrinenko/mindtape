//! Configuration file loading for `MindTape`.
//!
//! Supports TOML configuration with watched folders and database path.
//! Searches for `mindtape.toml` in the current directory, then
//! `~/.config/mindtape/config.toml`.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub watch: Vec<WatchEntry>,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WatchEntry {
    pub path: String,
    #[serde(default = "default_true")]
    pub recursive: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("config error: {0}")]
    Validation(String),
}

/// Load and parse a TOML config file.
///
/// # Errors
/// Returns `ConfigError` if the file cannot be read or contains invalid TOML.
pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let text = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&text)?;
    Ok(config)
}

/// Search for a config file in standard locations.
///
/// Checks (in order):
/// 1. `mindtape.toml` in the current working directory
/// 2. `~/.config/mindtape/config.toml` (XDG-compliant)
pub fn find_config() -> Option<PathBuf> {
    let cwd_config = PathBuf::from("mindtape.toml");
    if cwd_config.exists() {
        return Some(cwd_config);
    }

    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "mindtape") {
        let user_config = proj_dirs.config_dir().join("config.toml");
        if user_config.exists() {
            return Some(user_config);
        }
    }

    None
}

/// Expand a leading `~` to the user's home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    let expanded = shellexpand::tilde(path);
    PathBuf::from(expanded.as_ref())
}

/// Resolve the database path from config, with a default.
pub fn resolve_db_path(config: &Config) -> PathBuf {
    if let Some(ref db) = config.database {
        return expand_tilde(&db.path);
    }
    default_db_path()
}

/// Default database path: `~/.local/share/mindtape/index.db` (XDG-compliant).
pub fn default_db_path() -> PathBuf {
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "mindtape") {
        proj_dirs.data_dir().join("index.db")
    } else {
        PathBuf::from("index.db")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_config() {
        let toml = r#"
[database]
path = "~/.local/share/mindtape/index.db"

[[watch]]
path = "~/notes"
recursive = true

[[watch]]
path = "/tmp/tasks"
recursive = false
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.watch.len(), 2);
        assert_eq!(config.watch[0].path, "~/notes");
        assert!(config.watch[0].recursive);
        assert_eq!(config.watch[1].path, "/tmp/tasks");
        assert!(!config.watch[1].recursive);
        assert_eq!(
            config.database.as_ref().unwrap().path,
            "~/.local/share/mindtape/index.db"
        );
    }

    #[test]
    fn parse_minimal_config() {
        let toml = r#"
[[watch]]
path = "."
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.watch.len(), 1);
        assert!(config.watch[0].recursive); // default true
        assert!(config.database.is_none());
    }

    #[test]
    fn parse_empty_config() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.watch.is_empty());
        assert!(config.database.is_none());
    }

    #[test]
    fn parse_invalid_toml_errors() {
        let result: Result<Config, _> = toml::from_str("not valid [[[toml");
        assert!(result.is_err());
    }

    #[test]
    fn expand_tilde_with_home() {
        let path = expand_tilde("~/notes/tasks");
        // Should not start with ~/ anymore (expands to home dir)
        assert!(!path.to_string_lossy().starts_with("~/"));
        assert!(path.to_string_lossy().ends_with("/notes/tasks"));
    }

    #[test]
    fn expand_tilde_bare() {
        let path = expand_tilde("~");
        assert!(!path.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn expand_tilde_absolute_unchanged() {
        let path = expand_tilde("/tmp/tasks");
        assert_eq!(path, PathBuf::from("/tmp/tasks"));
    }

    #[test]
    fn expand_tilde_relative_unchanged() {
        let path = expand_tilde("relative/path");
        assert_eq!(path, PathBuf::from("relative/path"));
    }

    #[test]
    fn load_config_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("mindtape.toml");
        std::fs::write(
            &config_path,
            "[[watch]]\npath = \".\"\n",
        )
        .unwrap();
        let config = load_config(&config_path).unwrap();
        assert_eq!(config.watch.len(), 1);
    }

    #[test]
    fn load_config_missing_file_errors() {
        let result = load_config(Path::new("/nonexistent/config.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn resolve_db_path_from_config() {
        let config = Config {
            database: Some(DatabaseConfig {
                path: "/custom/path.db".to_string(),
            }),
            watch: vec![],
        };
        assert_eq!(resolve_db_path(&config), PathBuf::from("/custom/path.db"));
    }

    #[test]
    fn resolve_db_path_default() {
        let config = Config {
            database: None,
            watch: vec![],
        };
        let path = resolve_db_path(&config);
        assert!(path.to_string_lossy().ends_with("mindtape/index.db"));
    }
}
