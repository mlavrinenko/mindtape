//! Configuration file loading for `MindTape`.
//!
//! Supports TOML configuration with watched folders and database path.
//! Searches for `~/.config/mindtape/config.toml` (XDG-compliant).

use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Config {
    pub database: Option<DatabaseConfig>,
    #[serde(default)]
    pub watch: Vec<WatchEntry>,
    pub agenda: Option<AgendaConfig>,
}

/// Top-level agenda configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AgendaConfig {
    /// Global filter applied to all task sections (AND-ed with each section's filter).
    pub filter: Option<String>,
    #[serde(default)]
    pub sections: Vec<AgendaSection>,
}

/// A single section in the `agenda` output.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AgendaSection {
    pub name: String,
    /// Section kind: `"errors"` for file indexing errors, `"tasks"` (default) for task queries.
    #[serde(default = "default_tasks")]
    pub kind: String,
    /// Filter expression (evalexpr syntax, same as `list --filter`).
    pub filter: Option<String>,
    /// Sort specs (e.g. `["due:asc", "rank:desc"]`).
    #[serde(default)]
    pub sort: Vec<String>,
    /// Status filter: `"pending"` (default), `"done"`, or `"all"`.
    pub status: Option<String>,
    /// Limit output to N items.
    pub limit: Option<usize>,
    /// Deduplicate tasks across sections (default: true).
    /// When true, tasks shown in earlier sections are excluded from this one.
    pub deduplicate: Option<bool>,
}

fn default_tasks() -> String {
    "tasks".to_string()
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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

/// Search for a config file in the XDG config directory.
///
/// Checks `~/.config/mindtape/config.toml`.
pub fn find_config() -> Option<PathBuf> {
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

/// Merge multiple configs into one.
///
/// Watch entries are concatenated in order. The first config that specifies
/// a database path wins. An empty iterator returns an empty config.
pub fn merge_configs(configs: impl IntoIterator<Item = Config>) -> Config {
    let mut merged = Config {
        database: None,
        watch: Vec::new(),
        agenda: None,
    };
    for cfg in configs {
        merged.watch.extend(cfg.watch);
        if let Some(ag) = cfg.agenda {
            let dest = merged.agenda.get_or_insert_with(|| AgendaConfig {
                filter: None,
                sections: Vec::new(),
            });
            dest.sections.extend(ag.sections);
            if dest.filter.is_none() {
                dest.filter = ag.filter;
            }
        }
        if merged.database.is_none() {
            merged.database = cfg.database;
        }
    }
    merged
}

/// Load and merge multiple TOML config files.
///
/// # Errors
/// Returns `ConfigError` if any file cannot be read or parsed.
pub fn load_and_merge(paths: &[PathBuf]) -> Result<Config, ConfigError> {
    let mut configs = Vec::with_capacity(paths.len());
    for path in paths {
        configs.push(load_config(path)?);
    }
    Ok(merge_configs(configs))
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
        std::fs::write(&config_path, "[[watch]]\npath = \".\"\n").unwrap();
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
            agenda: None,
        };
        assert_eq!(resolve_db_path(&config), PathBuf::from("/custom/path.db"));
    }

    #[test]
    fn resolve_db_path_default() {
        let config = Config {
            database: None,
            watch: vec![],
            agenda: None,
        };
        let path = resolve_db_path(&config);
        assert!(path.to_string_lossy().ends_with("mindtape/index.db"));
    }

    #[test]
    fn merge_configs_concatenates_watch_entries() {
        let first = Config {
            database: None,
            watch: vec![WatchEntry {
                path: "~/a".into(),
                recursive: true,
            }],
            agenda: None,
        };
        let second = Config {
            database: None,
            watch: vec![WatchEntry {
                path: "~/b".into(),
                recursive: false,
            }],
            agenda: None,
        };
        let merged = merge_configs([first, second]);
        assert_eq!(merged.watch.len(), 2);
        assert_eq!(merged.watch[0].path, "~/a");
        assert_eq!(merged.watch[1].path, "~/b");
        assert!(merged.database.is_none());
    }

    #[test]
    fn merge_configs_first_database_wins() {
        let first = Config {
            database: Some(DatabaseConfig {
                path: "/first.db".into(),
            }),
            watch: vec![],
            agenda: None,
        };
        let second = Config {
            database: Some(DatabaseConfig {
                path: "/second.db".into(),
            }),
            watch: vec![],
            agenda: None,
        };
        let merged = merge_configs([first, second]);
        assert_eq!(merged.database.unwrap().path, "/first.db");
    }

    #[test]
    fn merge_configs_later_database_used_if_first_missing() {
        let first = Config {
            database: None,
            watch: vec![],
            agenda: None,
        };
        let second = Config {
            database: Some(DatabaseConfig {
                path: "/second.db".into(),
            }),
            watch: vec![],
            agenda: None,
        };
        let merged = merge_configs([first, second]);
        assert_eq!(merged.database.unwrap().path, "/second.db");
    }

    #[test]
    fn merge_configs_empty_iterator() {
        let merged = merge_configs(std::iter::empty());
        assert!(merged.watch.is_empty());
        assert!(merged.database.is_none());
    }

    #[test]
    fn load_and_merge_multiple_files() {
        let dir = tempfile::tempdir().unwrap();

        let path_a = dir.path().join("a.toml");
        std::fs::write(
            &path_a,
            "[database]\npath = \"/main.db\"\n\n[[watch]]\npath = \"/a\"\n",
        )
        .unwrap();

        let path_b = dir.path().join("b.toml");
        std::fs::write(&path_b, "[[watch]]\npath = \"/b\"\nrecursive = false\n").unwrap();

        let merged = load_and_merge(&[path_a, path_b]).unwrap();
        assert_eq!(merged.database.unwrap().path, "/main.db");
        assert_eq!(merged.watch.len(), 2);
        assert_eq!(merged.watch[0].path, "/a");
        assert_eq!(merged.watch[1].path, "/b");
        assert!(!merged.watch[1].recursive);
    }
}
