//! How a path reads as text. A path shown to people uses the native
//! separator, `\` on Windows, or the one `--path-separator` picked: the
//! `file` column, `--format path`, an error's place and the paths in a
//! message. A path a program reads or a file stores uses `/` on every
//! platform: the JSON and CSV `file` field, a link target, a lock entry.
//! git, rustc, ripgrep and fd draw the same line.
//!
//! [`display_path`] and [`slash_path`] are the only two ways a path becomes
//! text, and `clippy.toml` refuses `Path::display` elsewhere.

use std::path::{Component, MAIN_SEPARATOR, Path};
use std::sync::OnceLock;

/// The separator [`display_path`] writes, once [`init_path_separator`] set
/// one. Unset, it is the platform's own.
static SEPARATOR: OnceLock<char> = OnceLock::new();

/// Show every path in `separator` for the rest of the process. Call it once,
/// from `main`, before anything prints; a later call changes nothing.
pub fn init_path_separator(separator: char) {
    let _ = SEPARATOR.set(separator);
}

/// `path` as people read it: each separator written as the native one, or
/// the one [`init_path_separator`] set. On Windows, `/` and `\` both count as
/// separators, so a path joined from both comes out with one.
#[must_use]
pub fn display_path(path: impl AsRef<Path>) -> String {
    let separator = SEPARATOR.get().copied().unwrap_or(MAIN_SEPARATOR);
    with_separator(&path.as_ref().to_string_lossy(), separator, cfg!(windows))
}

/// `path` with its components joined by `/` on every platform: the form a
/// program reads and a file stores, such as a link target or the JSON `file`.
/// An absolute path keeps its one leading `/`, and a Windows prefix reads
/// `C:` or `//server/share`.
#[must_use]
pub fn slash_path(path: &Path) -> String {
    let mut text = String::new();
    // Whether the next named component follows another and needs a `/`.
    let mut follows = false;
    for part in path.components() {
        let name = part.as_os_str().to_string_lossy();
        match part {
            Component::Prefix(_) => text.push_str(&with_separator(&name, '/', true)),
            Component::RootDir => text.push('/'),
            Component::CurDir | Component::ParentDir | Component::Normal(_) => {
                if follows {
                    text.push('/');
                }
                text.push_str(&name);
            }
        }
        follows = matches!(
            part,
            Component::CurDir | Component::ParentDir | Component::Normal(_)
        );
    }
    text
}

/// `text` with each separator written as `separator`. `/` always counts as
/// one, and `\` only when `backslash` says so, as on Windows: elsewhere a file
/// name may hold it.
pub(crate) fn with_separator(text: &str, separator: char, backslash: bool) -> String {
    text.chars()
        .map(|c| {
            if c == '/' || (backslash && c == '\\') {
                separator
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "path_text_tests.rs"]
mod tests;
