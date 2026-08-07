#![allow(clippy::unwrap_used)]

use mindtape::test_support::bare_task;
use mindtape::{Config, Filter, Readiness, select};

use super::{FindArgs, desugar, quote, run_with};
use crate::ls::LsFlags;

fn args(terms: &[&str]) -> FindArgs {
    FindArgs {
        terms: terms.iter().map(|t| (*t).to_owned()).collect(),
        flags: LsFlags {
            format: crate::output::OutputFormat::Table,
            json: false,
            limit: None,
            sort: vec![],
            columns: None,
            interactive: false,
        },
    }
}

#[test]
fn desugar_wraps_one_term_in_contains() {
    assert_eq!(desugar(&["login".to_owned()]), "contains(\"login\")");
}

#[test]
fn desugar_conjoins_several_terms_with_and() {
    assert_eq!(
        desugar(&["login".to_owned(), "bug".to_owned()]),
        "contains(\"login\") && contains(\"bug\")"
    );
}

#[test]
fn quote_escapes_backslash_before_quote() {
    // Backslash must escape first: a literal `\"` in the term should come out
    // as `\\\"` (escaped backslash, escaped quote), not `\\"` (which would
    // close the string early).
    assert_eq!(quote(r#"a\"b"#), r#""a\\\"b""#);
}

#[test]
fn quote_plain_term_is_unchanged_inside_the_literal() {
    assert_eq!(quote("login"), "\"login\"");
}

/// The desugared expression must be the exact same string `contains()`
/// accepts directly — proves `find` adds no semantics of its own, only
/// syntax sugar over the filter language.
#[test]
fn desugared_expression_parses_and_matches_via_contains() {
    let mut login_task = bare_task("a");
    login_task.title = "fix login flow".to_owned();
    let mut bug_task = bare_task("b");
    bug_task.title = "squash a bug".to_owned();
    let mut both_task = bare_task("c");
    both_task.title = "login bug triage".to_owned();
    let mut neither_task = bare_task("d");
    neither_task.title = "unrelated cleanup".to_owned();

    let expr = desugar(&["login".to_owned(), "bug".to_owned()]);
    let filter = Filter::parse(&expr).unwrap();
    let tasks = vec![login_task, bug_task, both_task, neither_task];
    let kept = select(
        tasks,
        Some(&filter),
        &Config::default(),
        &Readiness::empty(),
    )
    .unwrap();

    let ids: Vec<&str> = kept.iter().map(|t| t.id.as_str()).collect();
    assert_eq!(ids, vec!["c"], "only the task matching both terms survives");
}

fn project(dir: &std::path::Path, files: &[(&str, &str)]) -> crate::util::Project {
    crate::test_util::make_project(dir, files, Config::default())
}

#[test]
fn run_with_a_single_term_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let proj = project(dir.path(), &[("x.typ", mindtape::test_support::TASK_SRC)]);
    assert!(run_with(&args(&["task"]), proj).is_ok());
}

#[test]
fn run_with_several_terms_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let proj = project(dir.path(), &[("x.typ", mindtape::test_support::TASK_SRC)]);
    assert!(run_with(&args(&["task", "x"]), proj).is_ok());
}

#[test]
fn run_with_forwards_the_format_flag() {
    let dir = tempfile::tempdir().unwrap();
    let proj = project(dir.path(), &[("x.typ", mindtape::test_support::TASK_SRC)]);
    let mut find_args = args(&["task"]);
    find_args.flags.format = crate::output::OutputFormat::Json;
    assert!(run_with(&find_args, proj).is_ok());
}
