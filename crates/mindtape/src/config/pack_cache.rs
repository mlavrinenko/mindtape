//! A `<source>:<pack>` entry of a declared source, resolved in the source's
//! tree in the cache: `<cache root>/<sha256>/`, named by the lock.
//!
//! The tree came from the network, so [`resolve_cached`] hashes it with
//! [`crate::config::pack_sources::hash`] and refuses it unless it hashes to
//! the lock's `sha256`, before it reads a file of it. The hash refuses a
//! symlink, so a tree that passes holds plain files and directories only. The
//! pack part of the entry takes a plain name, so `<pack>.toml` and
//! `<pack>/pack.toml` both stay inside the tree.

use std::path::{Path, PathBuf};

use crate::config::pack_registry::{PACK_FILE, PackOrigin};
use crate::config::pack_resolve::{
    PackScope, ResolvedPack, not_installed, read_pack_file, refuse_ambiguous,
};
use crate::config::pack_sources::{hash, is_plain_name};
use crate::config::{ConfigError, quote};
use crate::problem::Problem;
use crate::suggest::Unknown;

/// The extension of a pack file.
const TOML: &str = ".toml";

/// Resolve `pack` of the declared `source`, the parts of `entry`, from the
/// source's cached tree.
///
/// # Errors
/// Returns [`ConfigError::Invalid`] when `pack` is no plain name, the source
/// isn't in the cache, its tree doesn't hash to the lock's `sha256`, the tree
/// holds no such pack or holds it both as a file and as a directory, or the
/// pack fails to parse or declares a key a pack may not.
pub(super) fn resolve_cached(
    entry: &str,
    source: &str,
    pack: &str,
    scope: &PackScope<'_>,
) -> Result<ResolvedPack, ConfigError> {
    if !is_plain_name(pack) {
        return Err(ConfigError::invalid(format!(
            "extends: a pack name takes letters, digits, '-' and '_': {}",
            quote(entry)
        )));
    }
    let tree = verified_tree(source, scope)?;
    let dir = tree.join(pack);
    let file = tree.join(format!("{pack}{TOML}"));
    let (path, dir) = if dir.join(PACK_FILE).is_file() {
        refuse_ambiguous(&dir)?;
        (dir.join(PACK_FILE), Some(dir))
    } else if file.is_file() {
        (file, None)
    } else {
        return Err(unknown_cached_pack(entry, source, &tree));
    };
    let (table, text) = read_pack_file(&path, Some(entry))?;
    Ok(ResolvedPack {
        origin: PackOrigin::Cached {
            name: entry.to_owned(),
            tree,
            dir,
        },
        table,
        text,
    })
}

/// The cached tree of `source`, once it hashes to the lock's `sha256`.
fn verified_tree(source: &str, scope: &PackScope<'_>) -> Result<PathBuf, ConfigError> {
    let locked = scope.sources.lock.entries.get(source);
    let (Some(cache), Some(locked)) = (scope.cache, locked) else {
        return Err(not_installed(source));
    };
    let tree = cache.join(&locked.sha256);
    if !tree.is_dir() {
        return Err(not_installed(source));
    }
    let mismatch = |detail: String| {
        ConfigError::invalid(format!(
            "extends: pack source {} does not match the lock: {}{detail}",
            quote(source),
            quote(&tree.display().to_string())
        ))
        .with_hint("delete the directory, then run `mt packs install` to fetch it again")
    };
    match hash(&tree) {
        Ok(actual) if actual == locked.sha256 => Ok(tree.clone()),
        Ok(_) => Err(mismatch(String::new())),
        Err(error) => Err(mismatch(format!(": {error}"))),
    }
}

/// `entry` names no pack of the tree at `tree`: list the ones it holds.
fn unknown_cached_pack(entry: &str, source: &str, tree: &Path) -> ConfigError {
    let names: Vec<String> = tree_packs(tree)
        .into_iter()
        .map(|pack| format!("{source}:{pack}"))
        .collect();
    ConfigError::invalid(
        Problem::new(format!("extends: unknown pack: {}", quote(entry)))
            .with(Unknown::among(entry, names.iter().map(String::as_str))),
    )
}

/// The packs at the top of `tree`, sorted: each `<pack>.toml` file and each
/// `<pack>/` directory holding a `pack.toml`, whose name is plain.
fn tree_packs(tree: &Path) -> Vec<String> {
    let mut packs = Vec::new();
    for entry in std::fs::read_dir(tree).into_iter().flatten().flatten() {
        let path = entry.path();
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let pack = if path.is_dir() {
            path.join(PACK_FILE).is_file().then_some(name)
        } else {
            name.strip_suffix(TOML).map(str::to_owned)
        };
        packs.extend(pack.filter(|pack| is_plain_name(pack)));
    }
    packs.sort_unstable();
    packs.dedup();
    packs
}
