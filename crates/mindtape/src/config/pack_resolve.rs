//! Resolving one `extends` entry to a validated pack table: a path against
//! the extending layer's directory, any other entry as `<source>:<pack>`
//! against the embedded registry, and the shape check that keeps a pack to
//! rule arrays and `[dicts]`. [`crate::config::packs`] folds what this returns.

use std::path::Path;

use crate::config::ConfigError;
use crate::config::pack_registry::{EMBEDDED_SOURCE, PackOrigin};
use crate::problem::Problem;
use crate::suggest::Unknown;

/// One resolved pack: its provenance and validated table.
pub(super) struct ResolvedPack {
    pub(super) origin: PackOrigin,
    pub(super) table: toml::Table,
    /// The pack's source text, for the line of each rule's header.
    pub(super) text: String,
}

/// A path entry has a path separator somewhere in it. Any other entry is a
/// `<source>:<pack>` name, so `"tasks.toml"` with no separator is an unknown
/// pack, never a missing file.
fn is_path_entry(entry: &str) -> bool {
    entry.chars().any(std::path::is_separator)
}

/// Resolve one `extends` entry to its pack table, validating its shape.
pub(super) fn resolve_pack(
    entry: &str,
    base_dir: &Path,
    registry: &[(&str, &str)],
) -> Result<ResolvedPack, ConfigError> {
    if is_path_entry(entry) {
        let path = base_dir.join(entry);
        let text = std::fs::read_to_string(&path).map_err(|_| {
            ConfigError::invalid(format!(
                "extends: pack not found: {}",
                crate::config::quote(&path.display().to_string())
            ))
        })?;
        let table: toml::Table = toml::from_str(&text).map_err(|error| ConfigError::ParseIn {
            origin: path.display().to_string(),
            error,
        })?;
        validate_pack_shape(&table, &path.display().to_string())?;
        Ok(ResolvedPack {
            origin: PackOrigin::Path(path),
            table,
            text,
        })
    } else {
        let text = embedded_text(entry, registry)?;
        let table: toml::Table = toml::from_str(text).map_err(|error| ConfigError::ParseIn {
            origin: format!("pack {entry}"),
            error,
        })?;
        validate_pack_shape(&table, entry)?;
        Ok(ResolvedPack {
            origin: PackOrigin::Embedded(entry.to_owned()),
            table,
            text: text.to_owned(),
        })
    }
}

/// The text of the pack a `<source>:<pack>` entry names in `registry`, the
/// packs of the [`EMBEDDED_SOURCE`] source.
fn embedded_text<'r>(entry: &str, registry: &[(&'r str, &'r str)]) -> Result<&'r str, ConfigError> {
    let Some((source, pack)) = entry.split_once(':') else {
        return Err(unknown_pack_error(entry, registry));
    };
    if source != EMBEDDED_SOURCE {
        return Err(unknown_source_error(source));
    }
    registry
        .iter()
        .find(|(name, _)| *name == pack)
        .map(|(_, text)| *text)
        .ok_or_else(|| unknown_pack_error(entry, registry))
}

fn unknown_source_error(source: &str) -> ConfigError {
    ConfigError::invalid(
        Problem::new(format!(
            "extends: unknown pack source: {}",
            crate::config::quote(source)
        ))
        .with(Unknown::among(source, [EMBEDDED_SOURCE])),
    )
}

fn unknown_pack_error(entry: &str, registry: &[(&str, &str)]) -> ConfigError {
    let names: Vec<String> = registry
        .iter()
        .map(|(name, _)| format!("{EMBEDDED_SOURCE}:{name}"))
        .collect();
    ConfigError::invalid(
        Problem::new(format!(
            "extends: unknown pack: {}",
            crate::config::quote(entry)
        ))
        .with(Unknown::among(entry, names.iter().map(String::as_str))),
    )
}

/// A pack may declare only `[[tasks.rules]]`, `[[tasks.body.headers.rules]]`,
/// `[[tasks.body.content.rules]]`, `[[commits.rules]]` and `[dicts]` — nested
/// exactly that deep, nothing else, including `extends` itself, so resolution
/// stays one level deep.
fn validate_pack_shape(table: &toml::Table, pack_label: &str) -> Result<(), ConfigError> {
    reject_unknown_keys(table, &["tasks", "commits", "dicts"], pack_label)?;
    if let Some(commits) = table.get("commits").and_then(toml::Value::as_table) {
        reject_unknown_keys(commits, &["rules"], pack_label)?;
    }
    let Some(tasks) = table.get("tasks").and_then(toml::Value::as_table) else {
        return Ok(());
    };
    reject_unknown_keys(tasks, &["rules", "body"], pack_label)?;
    let Some(body) = tasks.get("body").and_then(toml::Value::as_table) else {
        return Ok(());
    };
    reject_unknown_keys(body, &["headers", "content"], pack_label)?;
    if let Some(headers) = body.get("headers").and_then(toml::Value::as_table) {
        reject_unknown_keys(headers, &["rules"], pack_label)?;
    }
    if let Some(content) = body.get("content").and_then(toml::Value::as_table) {
        reject_unknown_keys(content, &["rules"], pack_label)?;
    }
    Ok(())
}

fn reject_unknown_keys(
    table: &toml::Table,
    allowed: &[&str],
    pack_label: &str,
) -> Result<(), ConfigError> {
    for key in table.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(ConfigError::invalid(format!(
                "extends: pack {} may not declare: {}",
                crate::config::quote(pack_label),
                crate::config::quote(key)
            )));
        }
    }
    Ok(())
}
