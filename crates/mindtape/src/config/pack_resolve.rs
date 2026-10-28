//! Resolving one `extends` entry to a validated pack table: a path against
//! the extending layer's directory, to a file or to a directory pack
//! ([`resolve_dir_pack`]), any other entry as `<source>:<pack>` against the
//! embedded registry or, through [`crate::config::pack_cache`], a declared
//! source's cached tree, and the shape check that refuses the keys a pack may
//! not declare. [`crate::config::packs`] folds what this returns.

use std::path::{Path, PathBuf};

use crate::config::ConfigError;
use crate::config::pack_cache::{VerifiedTrees, resolve_cached};
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
    /// The directory the declared sources are cached in, or `None` when no
    /// source is installed.
    pub(super) cache: Option<&'a Path>,
    /// The cached trees this config load has already hashed.
    pub(super) verified: &'a VerifiedTrees,
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
        let (table, text) = read_pack_file(&path, None)?;
        Ok(ResolvedPack {
            origin: PackOrigin::Path(path),
            table,
            text,
        })
    } else {
        if let Some((source, pack)) = entry.split_once(':')
            && source != EMBEDDED_SOURCE
            && scope.project
            && scope.sources.declares(source)
        {
            return resolve_cached(entry, source, pack, scope);
        }
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
/// missing file.
fn canonical(path: &Path) -> PathBuf {
    crate::canonical(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Resolve the directory pack at `dir`: its `pack.toml`, validated like any
/// pack. The origin records the directory, so the fold can find its
/// `templates/`. A pack cached from a remote source resolves the same way.
///
/// # Errors
/// Returns [`ConfigError`] when `dir` holds no readable `pack.toml`, it
/// doesn't parse, or it declares a key a pack may not.
pub(super) fn resolve_dir_pack(dir: &Path) -> Result<ResolvedPack, ConfigError> {
    let (table, text) = read_pack_file(&dir.join(PACK_FILE), None)?;
    Ok(ResolvedPack {
        origin: PackOrigin::Dir(dir.to_path_buf()),
        table,
        text,
    })
}

/// Refuse a directory `dir` that sits beside a `<dir>.toml`: the entry could
/// mean either, so `mt` names both and picks neither.
pub(super) fn refuse_ambiguous(dir: &Path) -> Result<(), ConfigError> {
    let mut sibling = dir.as_os_str().to_owned();
    sibling.push(".toml");
    let sibling = PathBuf::from(sibling);
    if sibling.is_file() {
        return Err(ConfigError::invalid(format!(
            "extends: ambiguous pack: {} is a directory beside {}",
            crate::config::quote(&crate::display_path(dir)),
            crate::config::quote(&crate::display_path(&sibling))
        ))
        .with_hint("name the file with its .toml extension, or rename one of them"));
    }
    Ok(())
}

/// Read and validate the pack file at `path`, which errors name by `label`
/// when given, else by the path.
pub(super) fn read_pack_file(
    path: &Path,
    label: Option<&str>,
) -> Result<(toml::Table, String), ConfigError> {
    let text = std::fs::read_to_string(path).map_err(|_| {
        ConfigError::invalid(format!(
            "extends: pack not found: {}",
            crate::config::quote(&crate::display_path(path))
        ))
    })?;
    let table: toml::Table = toml::from_str(&text).map_err(|error| ConfigError::ParseIn {
        origin: crate::display_path(path),
        error,
    })?;
    let shown = crate::display_path(path);
    validate_pack_shape(&table, label.unwrap_or(&shown))?;
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

/// Why `source`, not [`EMBEDDED_SOURCE`], resolves no pack here: the project
/// declares no such source, or this layer isn't the project's, the only one
/// that may use one.
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
    ConfigError::invalid(format!(
        "extends: only the project config may use pack source: {quoted}"
    ))
    .with_hint("move the entry into the project's .mindtape/config.toml")
}

/// A declared `source` whose tree isn't in the cache.
pub(super) fn not_installed(source: &str) -> ConfigError {
    ConfigError::invalid(format!(
        "extends: pack source not installed: {}",
        crate::config::quote(source)
    ))
    .with_hint("run `mt packs install` to fetch the locked sources")
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
