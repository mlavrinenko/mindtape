//! Reading a `git ls-tree -r -z` listing into the blobs to write, and
//! refusing a path that a platform would write differently or outside the
//! tree. Every check runs on every platform, so a lock names the same tree
//! everywhere.
//!
//! A path is ASCII, so lowercasing it is the one case fold Windows and
//! macOS apply, and Unicode normalization never comes up. Each part is a
//! plain name: not empty, `.` or `..`, and nothing Windows refuses or
//! rewrites, such as `C:x`, `a\b`, a trailing dot or `CON`.

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use crate::config::pack_sources::FetchError;
use crate::config::pack_sources::git::git_error;

/// The names Windows keeps for devices, with or without an extension.
const DEVICES: [&str; 4] = ["con", "prn", "aux", "nul"];

/// The blobs of a `git ls-tree -r -z` listing, as object id and path. A
/// symlink, a submodule, or a path [`check_path`] refuses fails.
pub(super) fn parse_listing(listing: &[u8]) -> Result<Vec<(String, String)>, FetchError> {
    let mut blobs = Vec::new();
    let mut spellings = BTreeMap::new();
    for record in listing
        .split(|byte| *byte == 0)
        .filter(|record| !record.is_empty())
    {
        let record = String::from_utf8_lossy(record);
        let (meta, path) = record.split_once('\t').unwrap_or((&record, ""));
        let mut fields = meta.split(' ');
        match (fields.next(), fields.nth(1)) {
            (Some("120000"), _) => return Err(FetchError::unsupported("symlink", path)),
            (Some("160000"), _) => return Err(FetchError::unsupported("submodule", path)),
            (Some(_), Some(id)) if !path.is_empty() => {
                check_path(path)?;
                check_case(path, &mut spellings)?;
                blobs.push((id.to_owned(), path.to_owned()));
            }
            _ => return Err(git_error("ls-tree", record.as_bytes())),
        }
    }
    Ok(blobs)
}

/// Refuse a path that isn't ASCII, has an empty, `.` or `..` part, or a
/// part Windows can't hold.
fn check_path(path: &str) -> Result<(), FetchError> {
    if !path.is_ascii() {
        return Err(FetchError::unsupported("path that is not ASCII", path));
    }
    let parts = || path.split('/');
    if parts().any(|part| matches!(part, "" | "." | "..")) {
        return Err(FetchError::unsupported(
            "path with an empty, '.' or '..' part",
            path,
        ));
    }
    if parts().any(|part| !windows_can_hold(part)) {
        return Err(FetchError::unsupported("name Windows can't hold", path));
    }
    Ok(())
}

/// Whether Windows writes `name`, an ASCII path part, as given.
fn windows_can_hold(name: &str) -> bool {
    let bad_char = name.chars().any(|ch| {
        ch.is_ascii_control() || matches!(ch, '\\' | ':' | '<' | '>' | '"' | '|' | '?' | '*')
    });
    let stem = name.split('.').next().unwrap_or(name).to_ascii_lowercase();
    let device = DEVICES.contains(&stem.as_str()) || is_port(&stem);
    !bad_char && !device && !name.ends_with(['.', ' '])
}

/// `com0`-`com9` and `lpt0`-`lpt9`.
fn is_port(stem: &str) -> bool {
    let (Some(prefix), Some(digit)) = (stem.get(..3), stem.get(3..)) else {
        return false;
    };
    matches!(prefix, "com" | "lpt")
        && digit.len() == 1
        && digit.chars().all(|ch| ch.is_ascii_digit())
}

/// Refuse `path` when it, or a directory above it, lowercases to a path an
/// earlier entry spelled another way, since a case-insensitive file system
/// would write both as one.
fn check_case(path: &str, spellings: &mut BTreeMap<String, String>) -> Result<(), FetchError> {
    let prefixes = path
        .char_indices()
        .filter(|(_, ch)| *ch == '/')
        .map(|(at, _)| &path[..at])
        .chain([path]);
    for prefix in prefixes {
        let seen = spellings
            .entry(prefix.to_ascii_lowercase())
            .or_insert_with(|| prefix.to_owned());
        if seen != prefix {
            return Err(FetchError::unsupported(
                "path that differs only in case",
                path,
            ));
        }
    }
    Ok(())
}

/// `path`, a `/`-separated path from the listing, under `tree`, built only
/// from plain names, so no part can climb out of it.
pub(super) fn safe_join(tree: &Path, path: &str) -> Result<PathBuf, FetchError> {
    let leaves = || FetchError::unsupported("path with an empty, '.' or '..' part", path);
    let mut joined = tree.to_path_buf();
    for part in path.split('/') {
        let mut components = Path::new(part).components();
        match (components.next(), components.next()) {
            (Some(Component::Normal(name)), None) if name == part && windows_can_hold(part) => {
                joined.push(name);
            }
            _ => return Err(leaves()),
        }
    }
    if !joined.starts_with(tree) {
        return Err(leaves());
    }
    Ok(joined)
}

#[cfg(test)]
#[path = "listing_tests.rs"]
mod tests;
