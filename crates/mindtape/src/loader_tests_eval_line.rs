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
    assert_eq!(error.message, "eval error: expected comma");
    assert_eq!(error.line, Some(5));
    assert_eq!(error.place(), "tasks/t.typ:5");
}

#[test]
fn an_unexpected_argument_names_the_argument_line() {
    let error = load_error(
        "#import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(\n  title: \"T\",\n  titel: \"typo\",\n)\n",
    );
    assert_eq!(error.message, "eval error: unexpected argument: titel");
    assert_eq!(error.line, Some(4));
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
        line: None,
        message: "source harvested no task marker".to_owned(),
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
    typst_harvest::HarvestError::Eval(typst_harvest::EvalError {
        main: "tasks/t.typ".to_owned(),
        diagnostics: vec![typst_harvest::Diagnostic {
            message: "boom".to_owned(),
            location: location.map(|(path, line)| typst_harvest::Location {
                path: path.to_owned(),
                line,
                column: 1,
            }),
            trace: Vec::new(),
        }],
    })
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
    assert_eq!(error.message, "eval error: boom");
}
