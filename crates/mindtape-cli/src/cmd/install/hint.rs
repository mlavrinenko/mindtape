//! Load failures whose diagnostic names a symptom but never the remedy, and
//! the hints that name it.
//!
//! - Task files import `@local/mindtape`, but the machine has never run
//!   `mt install`. Typst reports it per file as `file not found (searched at
//!   <data dir>/typst/packages/local/mindtape/<version>/typst.toml)`, a path
//!   and not a remedy. Every task in the project fails the same way, so
//!   [`crate::util::report_load_problems`] folds them into the one
//!   [`prelude_diag`] that names the command and the config key.
//!   [`prelude_hints`] puts the same advice on a refused write's own error.
//! - A task body is Typst markup, so a literal `#` in prose (`Channel #N
//!   carries it`) opens an inline expression and the file fails with `unknown
//!   variable: N`. The message names a variable the reader never wrote and
//!   never mentions `\#`, so [`escaped_hash_hint`] says it.
//! - A misspelled status helper (`status: finished(…)`) fails with `unknown
//!   variable: finished`, so [`status_hint`] names the command that lists the
//!   statuses.
//!
//! [`RegionHints`] picks between the last two by where the error falls
//! against the `task.with(…)` call, and gives each once per run.

use mindtape::LoadError;
use mindtape::model::task_spans::CallRegion;

use crate::diag::{Command, Diag, Hint, Kind, Note};

/// The path segment Typst searched: the local-package directory for the
/// prelude. Matched instead of the diagnostic's wording, which is Typst's to
/// change; separators are normalised so this holds on Windows too.
const PRELUDE_PKG_PATH: &str = "packages/local/mindtape/";

/// What Typst searched when no data directory resolves at all (an empty
/// `$MINDTAPE_DATA_HOME`): the package spec itself, with no path.
const PRELUDE_SPEC: &str = "searched at @local/mindtape:";

/// Whether `message` is a load error about the unresolved prelude package.
///
/// Deliberately not narrowed to "file not found": a partially written or
/// version-mismatched package directory produces a different diagnostic naming
/// the same path, and `mt install` is the answer to those too.
pub(crate) fn names_prelude_package(message: &str) -> bool {
    message.replace('\\', "/").contains(PRELUDE_PKG_PATH) || message.contains(PRELUDE_SPEC)
}

/// The diagnostic Typst emits for an identifier that resolves to nothing —
/// which is what `#N` in prose is.
const UNKNOWN_VARIABLE: &str = "unknown variable";

/// Whether `message` is the unknown-variable diagnostic itself.
///
/// Anchored at the start rather than searched for: an error that merely quotes
/// the phrase, such as a task titled `unknown variable` or a nested diagnostic
/// naming another failure's text, is a different failure, and pointing its
/// author at `\#` sends them after a `#` that is not there.
fn is_unknown_variable(message: &str) -> bool {
    message.starts_with(UNKNOWN_VARIABLE)
}

/// The path Typst says it searched, from a message like `file not found
/// (searched at <path>)`, so the one prelude error can still say where.
fn searched_path(message: &str) -> Option<&str> {
    let (_, tail) = message.split_once("searched at ")?;
    let path = tail.strip_suffix(')').unwrap_or(tail);
    (!path.starts_with('@')).then_some(path)
}

/// The one error for a prelude that did not resolve, however many task files
/// failed on it: the package, where Typst looked (from the first failure's
/// `message`), the command that installs it, and the config key that pins a
/// copy instead.
pub(crate) fn prelude_diag(kind: Kind, message: &str) -> Diag {
    let diag = Diag::of(kind, "can't find the prelude package").value("@local/mindtape");
    let diag = match searched_path(message) {
        Some(path) => diag.note(Note::new("searched at").value(path)),
        None => diag,
    };
    prelude_hints(diag)
}

/// `diag` with the remedy for a prelude that does not resolve: the install
/// command, and the config key that pins a copy instead. For a writer that
/// refused a source because its import failed.
pub(crate) fn prelude_hints(diag: Diag) -> Diag {
    diag.hint(Hint::command(
        "install it once per machine",
        Command::mt_global().word("install"),
    ))
    .hint(Hint::new(
        "or pin a copy under [[typst.local-packages]] in the config",
    ))
}

/// The hint for a body whose prose was read as markup. Names both remedies,
/// because a run of `#`-heavy prose is better off in a code block than
/// escaped hash by hash.
fn escaped_hash_hint() -> Hint {
    Hint::new(
        "a literal # in a task body opens a Typst expression: write \\# or put the prose in a code block",
    )
}

/// The `task.with(…)` argument a status helper fills.
const STATUS_ARGUMENT: &str = "status";

/// The hint for a misspelled status helper, worded as `mt flip` words its
/// own refusal of an unknown status.
fn status_hint() -> Hint {
    Hint::run(
        Command::mt().word("flip").word("--list"),
        "to see the statuses",
    )
}

/// The region hints one run has given, so each rides on the first load error
/// that needs it and a project with many such errors reads it once.
#[derive(Debug, Default)]
pub(crate) struct RegionHints {
    hash: bool,
    status: bool,
}

impl RegionHints {
    /// The hint an `unknown variable` `error` gets the first time its region
    /// asks for one: [`escaped_hash_hint`] outside the `task.with(…)` call,
    /// [`status_hint`] in its `status:` argument. An error elsewhere in the
    /// call gets neither and uses neither up.
    pub(crate) fn hint_for(&mut self, error: &LoadError) -> Option<Hint> {
        if !is_unknown_variable(&error.message) {
            return None;
        }
        let (given, hint) = match &*error.region {
            CallRegion::Outside => (&mut self.hash, escaped_hash_hint()),
            CallRegion::Argument(name) if name == STATUS_ARGUMENT => {
                (&mut self.status, status_hint())
            }
            CallRegion::Argument(_) | CallRegion::Call => return None,
        };
        if *given {
            return None;
        }
        *given = true;
        Some(hint)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::{is_unknown_variable, names_prelude_package, prelude_diag};
    use crate::diag::{Kind, plain};

    /// Verbatim from `mt ls` against a project whose prelude is not installed.
    const REAL: &str = "file not found (searched at \
                        /home/u/.local/share/typst/packages/local/mindtape/0.2.0/typst.toml)";

    #[test]
    fn detects_the_real_diagnostic() {
        assert!(names_prelude_package(REAL));
    }

    #[test]
    fn detects_the_diagnostic_when_no_data_home_resolves() {
        let bare = "file not found (searched at @local/mindtape:0.2.0)";
        assert!(names_prelude_package(bare));
        assert_eq!(super::searched_path(bare), None);
    }

    #[test]
    fn detects_windows_separators() {
        let windows = REAL.replace('/', "\\");
        assert!(names_prelude_package(&windows));
    }

    #[test]
    fn ignores_unrelated_load_errors() {
        assert!(!names_prelude_package("unexpected argument: foo"));
        assert!(!names_prelude_package(
            "file not found (searched at /tmp/p/packages/local/other/1.0.0/typst.toml)"
        ));
    }

    #[test]
    fn detects_the_real_unknown_variable_diagnostic() {
        assert!(is_unknown_variable("unknown variable: N"));
    }

    /// The reason the match is anchored: an error that quotes the phrase
    /// mid-text is some other failure, and `\#` is not its remedy.
    #[test]
    fn ignores_the_phrase_mid_text() {
        assert!(!is_unknown_variable("cannot resolve unknown variable: N"));
        assert!(!is_unknown_variable(
            "unexpected argument: unknown variable"
        ));
    }

    #[test]
    fn the_prelude_error_names_the_package_the_command_and_the_key() {
        assert_eq!(
            plain(&prelude_diag(Kind::Warning, REAL)),
            "warning: can't find the prelude package: '@local/mindtape'\n\
             hint: install it once per machine: mt install\n\
             hint: or pin a copy under [[typst.local-packages]] in the config\n\
             note: searched at: '/home/u/.local/share/typst/packages/local/mindtape/0.2.0/typst.toml'\n"
        );
    }
}
