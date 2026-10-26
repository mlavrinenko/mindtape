//! Resolving one `extends` entry to a validated pack table: a path against
//! the extending layer's directory, to a file or to a directory pack
//! ([`resolve_dir_pack`], which a fetched pack reuses), any other entry as
//! `<source>:<pack>` against the embedded registry or a declared pack source,
//! and the shape check that refuses the keys a pack may not declare.
//! [`crate::config::packs`] folds what this returns.

use std::path::{Path, PathBuf};

use crate::config::ConfigError;
use crate::config::pack_registry::{EMBEDDED_SOURCE, PACK_FILE, PackOrigin};
use crate::config::pack_sources::PackSources;
use crate::config::packs_toml::get_path;
use crate::problem::Problem;
use crate::suggest::Unknown;

/// What a `<source>:<pack>` entry of one layer resolves against.
pub(super) struct PackScope<'a> {
    /// The packs of the [`EMBEDDED_SOURCE`] source.
    pub(super) registry: &'a [(&'a str, &'a str)],
    /// The project's declared sources and lock.
    pub(super) sources: &'a PackSources,
    /// Whether the layer is the project's own, the one layer that may use a
    /// declared source.
    pub(super) project: bool,
}

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
    scope: &PackScope<'_>,
) -> Result<ResolvedPack, ConfigError> {
    if is_path_entry(entry) {
        let path = canonical(&base_dir.join(entry));
        if path.is_dir() {
            let dir = path.components().collect::<PathBuf>();
            refuse_ambiguous(&dir)?;
            return resolve_dir_pack(&dir);
        }
        let (table, text) = read_pack_file(&path)?;
        Ok(ResolvedPack {
            origin: PackOrigin::Path(path),
            table,
            text,
        })
    } else {
        let text = embedded_text(entry, scope)?;
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

/// `path` absolute and free of `..` and symlinks, so two entries naming one
/// file name one pack, or `path` itself when it can't be resolved, such as a
/// missing file. Windows' verbatim prefix `\\?\` comes off, so the label
/// reads like any other path.
fn canonical(path: &Path) -> PathBuf {
    path.canonicalize()
        .map_or_else(|_| path.to_path_buf(), strip_verbatim)
}

/// `path` without a Windows verbatim prefix: `\\?\C:\x` becomes `C:\x`,
/// and `\\?\UNC\host\share` becomes `\\host\share`.
fn strip_verbatim(path: PathBuf) -> PathBuf {
    let Some(text) = path.to_str() else {
        return path;
    };
    match text.strip_prefix(r"\\?\") {
        Some(rest) => match rest.strip_prefix(r"UNC\") {
            Some(share) => PathBuf::from(format!(r"\\{share}")),
            None => PathBuf::from(rest),
        },
        None => path,
    }
}

/// Resolve the directory pack at `dir`: its `pack.toml`, validated like any
/// pack. The origin records the directory, so the fold can find its
/// `templates/`. A pack cached from a remote source resolves the same way.
///
/// # Errors
/// Returns [`ConfigError`] when `dir` holds no readable `pack.toml`, it
/// doesn't parse, or it declares a key a pack may not.
pub(super) fn resolve_dir_pack(dir: &Path) -> Result<ResolvedPack, ConfigError> {
    let (table, text) = read_pack_file(&dir.join(PACK_FILE))?;
    Ok(ResolvedPack {
        origin: PackOrigin::Dir(dir.to_path_buf()),
        table,
        text,
    })
}

/// Refuse a directory `dir` that sits beside a `<dir>.toml`: the entry could
/// mean either, so `mt` names both and picks neither.
fn refuse_ambiguous(dir: &Path) -> Result<(), ConfigError> {
    let mut sibling = dir.as_os_str().to_owned();
    sibling.push(".toml");
    let sibling = PathBuf::from(sibling);
    if sibling.is_file() {
        return Err(ConfigError::invalid(format!(
            "extends: ambiguous pack: {} is a directory beside {}",
            crate::config::quote(&dir.display().to_string()),
            crate::config::quote(&sibling.display().to_string())
        ))
        .with_hint("name the file with its .toml extension, or rename one of them"));
    }
    Ok(())
}

/// Read and validate the pack file at `path`.
fn read_pack_file(path: &Path) -> Result<(toml::Table, String), ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|_| {
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
    Ok((table, text))
}

/// The text of the pack a `<source>:<pack>` entry names among the packs of
/// the [`EMBEDDED_SOURCE`] source.
fn embedded_text<'r>(entry: &str, scope: &PackScope<'r>) -> Result<&'r str, ConfigError> {
    let registry = scope.registry;
    let Some((source, pack)) = entry.split_once(':') else {
        return Err(unknown_pack_error(entry, registry));
    };
    if source != EMBEDDED_SOURCE {
        return Err(source_error(source, scope));
    }
    registry
        .iter()
        .find(|(name, _)| *name == pack)
        .map(|(_, text)| *text)
        .ok_or_else(|| unknown_pack_error(entry, registry))
}

/// Why `source`, not [`EMBEDDED_SOURCE`], resolves no pack. A source the
/// project declares can't be read before it is installed, and only the
/// project layer may use one.
fn source_error(source: &str, scope: &PackScope<'_>) -> ConfigError {
    let quoted = crate::config::quote(source);
    if !scope.sources.declares(source) {
        let declared = scope.sources.sources.keys().filter(|_| scope.project);
        let known = std::iter::once(EMBEDDED_SOURCE).chain(declared.map(String::as_str));
        return ConfigError::invalid(
            Problem::new(format!("extends: unknown pack source: {quoted}"))
                .with(Unknown::among(source, known)),
        );
    }
    if scope.project {
        ConfigError::invalid(format!("extends: pack source not installed: {quoted}"))
            .with_hint("run `mt packs install` to fetch the locked sources")
    } else {
        ConfigError::invalid(format!(
            "extends: only the project config may use pack source: {quoted}"
        ))
        .with_hint("move the entry into the project's .mindtape/config.toml")
    }
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

/// The keys a pack may not declare, as TOML paths. `extends` and `disable`
/// would make a second level of packs, and `pack-sources` a second place to
/// declare a source; `project.code` is the project's own
/// name; `tasks.roots`, `templates.dirs` and `[typst]` name the project's
/// own tree. Every other key deep-merges like any config layer's.
const PACK_DENIED_KEYS: [&[&str]; 7] = [
    &["extends"],
    &["disable"],
    &["pack-sources"],
    &["project", "code"],
    &["tasks", "roots"],
    &["templates", "dirs"],
    &["typst"],
];

/// Refuse a pack declaring any of [`PACK_DENIED_KEYS`], naming the pack and
/// the dotted key.
fn validate_pack_shape(table: &toml::Table, pack_label: &str) -> Result<(), ConfigError> {
    match PACK_DENIED_KEYS
        .iter()
        .find(|path| get_path(table, path).is_some())
    {
        Some(path) => Err(ConfigError::invalid(format!(
            "extends: pack {} may not declare: {}",
            crate::config::quote(pack_label),
            crate::config::quote(&path.join("."))
        ))),
        None => Ok(()),
    }
}

#[cfg(test)]
#[path = "pack_resolve_tests.rs"]
mod tests;
