//! The framed tree hash a lock's `sha256` holds: every file under a
//! directory, sorted bytewise by its `/`-separated path, each framed by the
//! little-endian `u64` length of its path and of its bytes, so no two trees
//! frame to the same stream. A directory adds nothing of its own, so an
//! empty one is invisible, as it is to git.

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::config::pack_sources::FetchError;

/// The framed sha256 of every file under `dir`, in lowercase hex.
///
/// # Errors
/// Returns [`FetchError::Unsupported`] for a symlink or a path that is not
/// UTF-8, and [`FetchError::Io`] when a file can't be read.
pub fn hash(dir: &Path) -> Result<String, FetchError> {
    let files = files(dir)?;
    let mut hasher = Sha256::new();
    for path in &files {
        let bytes = std::fs::read(dir.join(path))?;
        frame(&mut hasher, path.as_bytes());
        frame(&mut hasher, &bytes);
    }
    Ok(crate::loader::cache::hex(&hasher.finalize()))
}

/// The `/`-separated path of every file under `dir`, sorted bytewise.
///
/// # Errors
/// As [`hash`], for a symlink, a path that is not UTF-8 or a failed read.
pub(super) fn files(dir: &Path) -> Result<Vec<String>, FetchError> {
    let mut files = Vec::new();
    collect(dir, "", &mut files)?;
    files.sort_unstable();
    Ok(files)
}

/// Feed `bytes` to `hasher`, led by its length.
fn frame(hasher: &mut Sha256, bytes: &[u8]) {
    let len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    hasher.update(len.to_le_bytes());
    hasher.update(bytes);
}

/// Push the `/`-separated path of every file under `dir`, each led by
/// `prefix`.
fn collect(dir: &Path, prefix: &str, files: &mut Vec<String>) -> Result<(), FetchError> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            return Err(FetchError::unsupported(
                "path that is not UTF-8",
                &format!("{prefix}{}", name.to_string_lossy()),
            ));
        };
        let path = format!("{prefix}{name}");
        let kind = entry.file_type()?;
        if kind.is_dir() {
            collect(&entry.path(), &format!("{path}/"), files)?;
        } else if kind.is_file() {
            files.push(path);
        } else {
            return Err(FetchError::unsupported("symlink", &path));
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "hash_tests.rs"]
mod tests;
