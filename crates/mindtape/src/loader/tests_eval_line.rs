//! Which line of a task file a load error names: the line of the first
//! diagnostic that lands in the task file, walking its trace when Typst raised
//! it inside the prelude, so the reader never sees a prelude line.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use super::tests::{setup, write, write_pkg};
use super::{LoadError, load_project};
use crate::config::Config;

use crate::test_support::PRELUDE;

/// Load `tasks/t.typ` holding `source` against the real prelude and return its
/// one load error.
fn load_error(source: &str) -> LoadError {
    let dir = setup();
    write_pkg(dir.path(), "lib", "mindtape", "0.2.0", PRELUDE);
    write(dir.path(), "tasks/t.typ", source);
    let mut errors = load_project(dir.path(), &Config::default()).errors;
    assert_eq!(errors.len(), 1, "{errors:#?}");
    errors.remove(0)
}

#[test]
fn a_syntax_error_names_its_line() {
    let error = load_error(
        "#import \"@local/mindtape:0.2.0\": *\n\
         \n\
         #show: task.with(\n  \
           // a comment\n  \
           title: \"Fix the login redirect\"\n  \
           assignee: \"ada\",\n\
         )\n",
    );
    assert_eq!(error.message, "expected comma");
    assert_eq!(error.line, Some(5));
    assert_eq!(error.column, Some(34));
    assert_eq!(error.place(), "tasks/t.typ:5:34");
}

#[test]
fn an_unexpected_argument_names_the_argument_line() {
    let error = load_error(
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(\n  title: \"T\",\n  titel: \"typo\",\n)\n",
    );
    assert_eq!(error.message, "unexpected argument: titel");
    assert_eq!(error.line, Some(4));
    assert_eq!(error.column, Some(3));
}

#[test]
fn an_assert_inside_the_prelude_names_the_task_files_call() {
    let error = load_error(
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(\n  title: \"T\",\n  \
           links: depends-on(link(\"a.typ\")),\n)\n",
    );
    assert!(
        error.message.contains("`depends-on()`"),
        "{}",
        error.message
    );
    assert_eq!(error.line, Some(4));
}

#[test]
fn an_error_with_no_line_names_the_file_alone() {
    let error = LoadError {
        file: "tasks/t.typ".into(),
        message: "source harvested no task marker".to_owned(),
        ..LoadError::default()
    };
    assert_eq!(error.place(), "tasks/t.typ");
}

/// Typst leaves a show-everything rule out of the trace, so an assert the
/// prelude's `task` raises reaches no line of the task file. The line of the
/// `task.with(…)` call that applied it stands in.
#[test]
fn an_assert_inside_the_show_rule_names_the_task_call() {
    let error = load_error(
        "#import \"@local/mindtape:0.2.0\": *\n\
         \n\
         #show: task.with(\n  \
           title: \"T\",\n  \
           links: depends-on(\"a.typ\"),\n    \
           related(\"b.typ\")[why],\n\
         )\n",
    );
    assert!(
        error.message.contains("no positional args"),
        "{}",
        error.message
    );
    assert_eq!(error.line, Some(3));
}

/// An eval error built by hand: one diagnostic, located at `location`.
fn eval_error(location: Option<(&str, usize)>) -> typst_harvest::HarvestError {
    typst_harvest::HarvestError::Eval {
        error: typst_harvest::EvalError {
            main: "tasks/t.typ".to_owned(),
            diagnostics: vec![typst_harvest::Diagnostic {
                severity: typst_harvest::Severity::Error,
                message: "boom".to_owned(),
                hints: vec![typst_harvest::Hint {
                    message: "try less".to_owned(),
                    location: None,
                }],
                location: location.map(|(path, line)| typst_harvest::Location {
                    path: path.to_owned(),
                    line,
                    column: 1,
                }),
                trace: Vec::new(),
            }],
        },
        warnings: Vec::new(),
    }
}

const CALL_ON_LINE_2: &str = "#import \"x.typ\": *\n#show: task.with(title: \"T\")\n";

#[test]
fn only_an_error_raised_elsewhere_falls_back_to_the_task_call() {
    let line = |err| LoadError::harvest("tasks/t.typ".into(), &err, CALL_ON_LINE_2).line;
    assert_eq!(line(eval_error(Some(("tasks/t.typ", 7)))), Some(7));
    assert_eq!(line(eval_error(Some(("lib.typ", 9)))), Some(2));
    assert_eq!(line(eval_error(None)), None);
    assert_eq!(
        line(typst_harvest::HarvestError::File("gone".to_owned())),
        None
    );
}

#[test]
fn an_error_raised_elsewhere_in_a_file_with_no_task_call_names_no_line() {
    let error = LoadError::harvest(
        "tasks/t.typ".into(),
        &eval_error(Some(("lib.typ", 9))),
        "= Just prose\n",
    );
    assert_eq!(error.line, None);
    assert_eq!(error.message, "boom");
    assert_eq!(error.hints, ["try less"]);
}

/// Typst's hints ride on the error, in its words, and the column is where
/// the diagnostic starts.
#[test]
fn an_error_keeps_typsts_hints() {
    let error = load_error(
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"T\")\n\
         #let a = 1\n\
         #a-b\n",
    );
    assert_eq!(error.message, "unknown variable: a-b");
    assert_eq!((error.line, error.column), (Some(4), Some(2)));
    assert_eq!(error.hints.len(), 1, "{:?}", error.hints);
    assert!(error.hints[0].contains("subtraction"), "{:?}", error.hints);
}

/// A warning on a file that loads is reported with its place.
#[test]
fn a_warning_on_a_file_that_loads_is_kept() {
    let dir = setup();
    write_pkg(dir.path(), "lib", "mindtape", "0.2.0", PRELUDE);
    write(
        dir.path(),
        "tasks/t.typ",
        "<orphan>\n\
         #import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"T\")\n",
    );
    let outcome = load_project(dir.path(), &Config::default());
    assert!(outcome.errors.is_empty(), "{:?}", outcome.errors);
    assert_eq!(outcome.warnings.len(), 1, "{:?}", outcome.warnings);
    let warning = &outcome.warnings[0];
    assert_eq!(
        warning.message,
        "label `<orphan>` is not attached to anything"
    );
    assert_eq!(warning.place(), "tasks/t.typ:1:1");
}
