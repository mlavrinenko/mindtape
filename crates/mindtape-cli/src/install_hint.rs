//! Load failures whose diagnostic names a symptom but never the remedy, and
//! the one-line hints that name it.
//!
//! Two of them today:
//!
//! - Task files import `@local/mindtape`, but the machine has never run
//!   `mt install`. Typst reports it per file as `file not found (searched at
//!   <data dir>/typst/packages/local/mindtape/<version>/typst.toml)` — a path,
//!   not a remedy. Every task in the project fails the same way, so `mt ls`
//!   prints a wall of warnings above an empty table and still exits 0, and
//!   nothing names the command that fixes it. A collaborator cloning a
//!   `MindTape` repo hits this before anything else.
//! - A task body is Typst markup, so a literal `#` in prose (`Channel #N
//!   carries it`) opens an inline expression and the file fails with `unknown
//!   variable: N`. The message names the variable the reader never wrote and
//!   never mentions `\#`, so the remedy has to be discovered rather than read.
//!
//! Every hint is derived once from the whole [`LoadOutcome`] rather than per
//! error, so a 2000-task backlog reports each once — both failures are
//! project-wide by construction, and repeating one per file buries the tasks
//! that failed for some other reason.

use mindtape::LoadOutcome;

/// Printed when the prelude cannot be resolved. Names the command, says it is
/// per-machine so nobody re-runs it per project, and names the per-project
/// alternative for a checkout that carries its own prelude.
const HINT: &str = "note: `@local/mindtape` is not installed — run `mt install` \
                    (once per machine) or pin the prelude under \
                    `[[typst.local-packages]]`, then try again";

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

/// The hint for one load failure, or `None` when the prelude resolved.
///
/// For a single candidate that failed on its own — `mt add` in a project with
/// no tasks yet, whose load outcome holds no error to derive the hint from.
#[must_use]
pub fn failure_prelude_hint(message: &str) -> Option<&'static str> {
    names_prelude_package(message).then_some(HINT)
}

/// The hint for a batch of failures, once: `Some` when any message names the
/// prelude. For a writer that refuses several entries at once (`mt import`,
/// `mt recur`, `mt mv`) and prints the note after their lines.
#[must_use]
pub fn batch_prelude_hint<'a>(messages: impl IntoIterator<Item = &'a str>) -> Option<&'static str> {
    messages.into_iter().find_map(failure_prelude_hint)
}

/// Whether `message` is a load error about the unresolved prelude package.
///
/// Deliberately not narrowed to "file not found": a partially written or
/// version-mismatched package directory produces a different diagnostic naming
/// the same path, and `mt install` is the answer to those too.
fn names_prelude_package(message: &str) -> bool {
    message.replace('\\', "/").contains(PRELUDE_PKG_PATH)
}

/// Printed when a body's prose was read as markup. Names both remedies,
/// because a run of `#`-heavy prose is better off in a code block than
/// escaped hash by hash.
const HASH_HINT: &str = "note: a literal `#` in a task body opens a Typst expression — write it \
                         `\\#`, or put the prose in a code block";

/// The prefix `Candidate` puts on every Typst diagnostic it turns into a
/// [`mindtape::LoadError`]; stripped before matching so the hint keys off the
/// diagnostic's own opening words, not that framing.
const EVAL_PREFIX: &str = "eval error: ";

/// The diagnostic Typst emits for an identifier that resolves to nothing —
/// which is what `#N` in prose is.
const UNKNOWN_VARIABLE: &str = "unknown variable";

/// The hint to print for this load outcome, or `None` when nothing failed on
/// an unbound identifier.
///
/// Sibling of [`missing_prelude_hint`] and derived the same way: one value for
/// the whole outcome, so the hint appears once however many bodies tripped.
#[must_use]
pub fn escaped_hash_hint(outcome: &LoadOutcome) -> Option<&'static str> {
    outcome
        .errors
        .iter()
        .any(|error| is_unknown_variable(&error.message))
        .then_some(HASH_HINT)
}

/// Whether `message` is the unknown-variable diagnostic itself.
///
/// Anchored at the start rather than searched for: an error that merely quotes
/// the phrase — a task titled `unknown variable`, a nested diagnostic naming
/// another failure's text — is a different failure, and pointing its author at
/// `\#` sends them after a `#` that is not there.
fn is_unknown_variable(message: &str) -> bool {
    message
        .strip_prefix(EVAL_PREFIX)
        .unwrap_or(message)
        .starts_with(UNKNOWN_VARIABLE)
}

/// Every hint this outcome earns, in the order they are printed.
///
/// The two are independent — a project can have both an uninstalled prelude
/// and an unescaped `#` — so this is a list, not a first-match.
pub fn load_hints(outcome: &LoadOutcome) -> impl Iterator<Item = &'static str> {
    [missing_prelude_hint(outcome), escaped_hash_hint(outcome)]
        .into_iter()
        .flatten()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::PathBuf;

    use mindtape::{LoadError, LoadOutcome};

    use super::{
        HINT, batch_prelude_hint, escaped_hash_hint, failure_prelude_hint, is_unknown_variable,
        load_hints, missing_prelude_hint, names_prelude_package,
    };

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
            "eval error: unexpected argument: foo"
        ));
        assert!(!names_prelude_package(
            "eval error: file not found (searched at /tmp/p/packages/local/other/1.0.0/typst.toml)"
        ));
    }

    /// One candidate's own failure earns the hint only when it names the
    /// prelude — `mt add` in a project with no tasks to derive it from.
    #[test]
    fn a_single_failure_hints_only_when_it_names_the_prelude() {
        assert_eq!(failure_prelude_hint(REAL), Some(HINT));
        assert_eq!(
            failure_prelude_hint("eval error: unexpected argument: foo"),
            None
        );
    }

    #[test]
    fn a_batch_hints_once_when_any_failure_names_the_prelude() {
        assert_eq!(
            batch_prelude_hint(["eval error: unexpected argument: foo", REAL, REAL]),
            Some(HINT)
        );
        assert_eq!(
            batch_prelude_hint(["eval error: unexpected argument: foo"]),
            None
        );
        assert_eq!(batch_prelude_hint([]), None);
    }

    #[test]
    fn no_hint_when_nothing_failed() {
        assert_eq!(missing_prelude_hint(&LoadOutcome::default()), None);
    }

    #[test]
    fn no_hint_for_other_failures() {
        assert_eq!(
            missing_prelude_hint(&outcome(&["eval error: unexpected argument: foo"])),
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
            missing_prelude_hint(&outcome(&["eval error: unexpected argument: foo", REAL]))
                .is_some()
        );
    }

    /// Verbatim from `mt ls` over a body reading `Channel #N carries it.`
    const UNBOUND: &str = "eval error: unknown variable: N";

    #[test]
    fn detects_the_real_unknown_variable_diagnostic() {
        assert!(is_unknown_variable(UNBOUND));
        assert!(escaped_hash_hint(&outcome(&[UNBOUND])).is_some());
    }

    /// The prefix is `Candidate`'s framing, not Typst's; a diagnostic reaching
    /// the outcome without it is the same failure.
    #[test]
    fn detects_the_bare_diagnostic() {
        assert!(is_unknown_variable("unknown variable: N"));
    }

    /// The reason the match is anchored: an error that quotes the phrase
    /// mid-text is some other failure, and `\#` is not its remedy.
    #[test]
    fn ignores_the_phrase_mid_text() {
        assert!(!is_unknown_variable(
            "eval error: cannot resolve unknown variable: N"
        ));
        assert_eq!(
            escaped_hash_hint(&outcome(&[
                "eval error: unexpected argument: unknown variable"
            ])),
            None
        );
    }

    #[test]
    fn no_hash_hint_when_nothing_failed() {
        assert_eq!(escaped_hash_hint(&LoadOutcome::default()), None);
    }

    /// Same once-per-run property the prelude hint has.
    #[test]
    fn one_hash_hint_however_many_bodies_failed() {
        let many = vec![UNBOUND; 2000];
        assert_eq!(
            load_hints(&outcome(&many)).collect::<Vec<_>>(),
            vec![super::HASH_HINT]
        );
    }

    /// Both failures at once yield both hints — neither suppresses the other.
    #[test]
    fn both_hints_coexist() {
        assert_eq!(
            load_hints(&outcome(&[REAL, UNBOUND])).collect::<Vec<_>>(),
            vec![super::HINT, super::HASH_HINT]
        );
    }

    #[test]
    fn no_hints_for_an_unrelated_failure() {
        assert_eq!(
            load_hints(&outcome(&["eval error: unexpected argument: foo"])).count(),
            0
        );
    }
}
