//! How the command line chose its project, restated for a hint to paste.
//!
//! A single-project `@name` is rewritten into `-w <path>` before clap parses
//! (`cmd::at`), so by the time a command builds a hint for the next step the
//! name is gone. This module remembers what the reader typed: the `@name`, or
//! else the `-w` value as typed. `$MINDTAPE_WORKTREE` is left out: the
//! environment carries over when the hint is pasted, and restating it would pin
//! a path the reader may have since changed.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::diag::shell_word;

static AT_NAME: OnceLock<String> = OnceLock::new();
static FLAG: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Record the single-project `@name` the command line led with. Call once from
/// the desugar; later calls are ignored.
pub fn init_at_name(name: &str) {
    let _ = AT_NAME.set(name.to_owned());
}

/// Record the `-w`/`--worktree` value as typed. Call once from `main`; later
/// calls are ignored.
pub fn init_flag(flag: Option<PathBuf>) {
    let _ = FLAG.set(flag);
}

/// The words after `mt` that select this run's project (`@sn`, `-w ../x`), or
/// `None` when the run discovered its project from the current directory.
#[must_use]
pub fn hint() -> Option<String> {
    words(AT_NAME.get().map(String::as_str), FLAG.get()?.as_deref())
}

/// The pure core of [`hint`]: the `@name` wins, because the desugar made the
/// flag out of it.
fn words(at_name: Option<&str>, flag: Option<&Path>) -> Option<String> {
    match (at_name, flag) {
        (Some(name), _) => Some(format!("@{name}")),
        (None, Some(path)) => Some(format!("-w {}", shell_word(&path.to_string_lossy()))),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_name_wins_over_the_flag_it_became() {
        let flag = Path::new("/p/sn");
        assert_eq!(words(Some("sn"), Some(flag)).as_deref(), Some("@sn"));
    }

    #[test]
    fn a_flag_is_restated_as_typed() {
        assert_eq!(
            words(None, Some(Path::new("../x"))).as_deref(),
            Some("-w ../x")
        );
    }

    #[test]
    fn a_flag_that_needs_quoting_is_quoted() {
        let quoted = words(None, Some(Path::new("my proj")));
        assert_eq!(quoted.as_deref(), Some("-w 'my proj'"));
    }

    #[test]
    fn no_selector_means_no_words() {
        assert_eq!(words(None, None), None);
    }
}
