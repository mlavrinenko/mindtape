//! The one first-run failure that looks like a broken tool: task files import
//! `@local/mindtape`, but the machine has never run `mt install`.
//!
//! Typst reports it per file as `file not found (searched at
//! <data dir>/typst/packages/local/mindtape/<version>/typst.toml)` — a path,
//! not a remedy. Every task in the project fails the same way, so `mt ls`
//! prints a wall of warnings above an empty table and still exits 0, and
//! nothing names the command that fixes it. A collaborator cloning a
//! `MindTape` repo hits this before anything else.
//!
//! The hint is derived once from the whole [`LoadOutcome`] rather than per
//! error, so a 2000-task backlog reports it once — the failure is machine-wide
//! by construction, and repeating it per file buries the tasks that failed for
//! some other reason.

use mindtape::LoadOutcome;

/// Printed when the prelude cannot be resolved. Names the command, and says it
/// is per-machine so nobody re-runs it per project.
const HINT: &str = "note: `@local/mindtape` is not installed — run `mt install` \
                    (once per machine), then try again";

/// The path segment Typst searched: the local-package directory for the
/// prelude. Matched instead of the diagnostic's wording, which is Typst's to
/// change; separators are normalised so this holds on Windows too.
const PRELUDE_PKG_PATH: &str = "packages/local/mindtape/";

/// The hint to print for this load outcome, or `None` when the prelude
/// resolved.
///
/// Returning a single value rather than one per error is what makes
/// "once per run" structural.
#[must_use]
pub fn missing_prelude_hint(outcome: &LoadOutcome) -> Option<&'static str> {
    outcome
        .errors
        .iter()
        .any(|error| names_prelude_package(&error.message))
        .then_some(HINT)
}

/// Whether `message` is a load error about the unresolved prelude package.
///
/// Deliberately not narrowed to "file not found": a partially written or
/// version-mismatched package directory produces a different diagnostic naming
/// the same path, and `mt install` is the answer to those too.
fn names_prelude_package(message: &str) -> bool {
    message.replace('\\', "/").contains(PRELUDE_PKG_PATH)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::PathBuf;

    use mindtape::{LoadError, LoadOutcome};

    use super::{missing_prelude_hint, names_prelude_package};

    /// Verbatim from `mt ls` against a project whose prelude is not installed.
    const REAL: &str = "eval error: file not found (searched at \
                        /home/u/.local/share/typst/packages/local/mindtape/0.2.0/typst.toml)";

    fn outcome(messages: &[&str]) -> LoadOutcome {
        LoadOutcome {
            errors: messages
                .iter()
                .enumerate()
                .map(|(i, message)| LoadError {
                    file: PathBuf::from(format!("tasks/t{i}.typ")),
                    message: (*message).to_owned(),
                })
                .collect(),
            ..LoadOutcome::default()
        }
    }

    #[test]
    fn detects_the_real_diagnostic() {
        assert!(names_prelude_package(REAL));
    }

    #[test]
    fn detects_windows_separators() {
        let windows = REAL.replace('/', "\\");
        assert!(names_prelude_package(&windows));
    }

    #[test]
    fn ignores_unrelated_load_errors() {
        assert!(!names_prelude_package(
            "eval error: unknown task slot `foo`"
        ));
        assert!(!names_prelude_package(
            "eval error: file not found (searched at /tmp/p/packages/local/other/1.0.0/typst.toml)"
        ));
    }

    #[test]
    fn no_hint_when_nothing_failed() {
        assert_eq!(missing_prelude_hint(&LoadOutcome::default()), None);
    }

    #[test]
    fn no_hint_for_other_failures() {
        assert_eq!(
            missing_prelude_hint(&outcome(&["eval error: unknown task slot `foo`"])),
            None
        );
    }

    /// The whole point: a backlog where every file failed yields one hint.
    #[test]
    fn one_hint_however_many_files_failed() {
        let many = vec![REAL; 2000];
        assert!(missing_prelude_hint(&outcome(&many)).is_some());
    }

    /// A mixed outcome still hints — the unrelated failure may well be a
    /// consequence of the missing prelude.
    #[test]
    fn hints_when_only_some_errors_name_the_package() {
        assert!(
            missing_prelude_hint(&outcome(&["eval error: unknown task slot `foo`", REAL]))
                .is_some()
        );
    }
}
