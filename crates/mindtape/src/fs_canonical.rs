//! The one door for resolving a path to its canonical form, and for showing
//! a path relative to a root that may be spelled another way.
//!
//! `std::fs::canonicalize` returns a verbatim path on Windows, such as
//! `\\?\C:\Users\ada\tasks`, which a user should never read. [`canonical`]
//! goes through `dunce`, which drops the prefix whenever the plain form names
//! the same file.
//!
//! A canonical path also loses a symlink, or on Windows a short `RUNNER~1`
//! name, that the caller's root still holds. So a bare `strip_prefix` of a
//! canonical path against that root fails, and the user reads an absolute
//! path. [`relative_to`] retries against the canonical root.
//!
//! `clippy.toml`'s `disallowed-methods` bans `Path::canonicalize` and
//! `std::fs::canonicalize` everywhere else in the tree.

use std::io;
use std::path::{Path, PathBuf};

/// `path` absolute, with every `..` and symlink resolved, and with no
/// Windows verbatim prefix.
///
/// # Errors
/// Returns the I/O error when `path` does not exist or can't be read.
pub fn canonical(path: &Path) -> io::Result<PathBuf> {
    dunce::canonicalize(path)
}

/// `path` relative to `root` when it sits under it, else `path` unchanged.
/// A relative `path` is taken as already shown, never resolved against the
/// working directory.
///
/// Both are tried as given first, then both in canonical form, so a root
/// reached through a symlink or a short name still matches a path that was
/// resolved.
#[must_use]
pub fn relative_to(path: &Path, root: &Path) -> PathBuf {
    if let Ok(rel) = path.strip_prefix(root) {
        return rel.to_path_buf();
    }
    if !path.is_absolute() {
        return path.to_path_buf();
    }
    let Ok(root) = canonical(root) else {
        return path.to_path_buf();
    };
    let resolved = canonical(path).unwrap_or_else(|_| path.to_path_buf());
    resolved
        .strip_prefix(&root)
        .map_or_else(|_| path.to_path_buf(), Path::to_path_buf)
}

#[cfg(test)]
#[path = "fs_canonical_tests.rs"]
mod tests;
