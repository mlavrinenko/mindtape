//! The one write primitive every authored file goes through
//! (`truth/tasks/write-every-authored-file-through-one-atomic-writer.typ`).
//!
//! `std::fs::write` truncates then writes: a process killed between the two
//! (Ctrl-C, OOM, a full disk) leaves an empty or torn file, and a concurrent
//! reader (tinymist, a hook's `mt ls`, an editor reload) can observe the gap.
//! [`write_atomic`] instead writes a sibling temp file, fsyncs it, then
//! renames it over the target — a killed writer leaves the old file whole,
//! never a half-written one.
//!
//! `std::fs::rename` replaces an existing destination on Windows the same as
//! on Unix (`MoveFileExW` with `MOVEFILE_REPLACE_EXISTING`), so this needs no
//! platform branch.
//!
//! `clippy.toml`'s `disallowed-methods` bans `std::fs::write` and
//! `std::fs::File::create` everywhere else in the tree; the single call below
//! carries the only `#[allow]`, so a new raw write fails `just check` naming
//! this function instead.

use std::fs::{self, File};
use std::io::{self, Write as _};
use std::path::Path;

/// Write `bytes` to `path` atomically.
///
/// Creates a uniquely-named temp file beside `path` (same directory, so the
/// final rename cannot cross a filesystem boundary), writes and fsyncs it,
/// then renames it over `path`. A failure at any step removes the temp file
/// and leaves `path` exactly as it was.
///
/// # Errors
/// Returns an error when the temp file cannot be created, written, fsynced,
/// or renamed into place.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let dir = match path.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        _ => Path::new("."),
    };
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let tmp = dir.join(format!(".{name}.{}.tmp", uuid::Uuid::new_v4()));

    let result = write_and_rename(&tmp, path, bytes);
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

/// The write-fsync-rename sequence, isolated so [`write_atomic`] can clean up
/// the temp file on any failure with one `if result.is_err()` rather than a
/// `?` at each step.
fn write_and_rename(tmp: &Path, path: &Path, bytes: &[u8]) -> io::Result<()> {
    #[allow(
        clippy::disallowed_methods,
        reason = "the sanctioned raw write inside write_atomic itself; every other call site \
                   goes through this function instead"
    )]
    let mut file = File::create(tmp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(tmp, path)
}

#[cfg(test)]
#[path = "fs_atomic_tests.rs"]
mod tests;
