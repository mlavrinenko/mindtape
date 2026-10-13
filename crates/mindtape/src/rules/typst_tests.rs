#![allow(clippy::unwrap_used)]

use super::{CompiledCheck, validate_load};
use crate::config::Config;
use crate::model::Task;
use crate::parse_config;
use crate::rules::CheckSpec;
use crate::test_support::tempdir;
use crate::test_support::{bare_task, write_bare_project};

/// A `check` spec naming `function` in `check.typ` at the project root, with
/// no `field` — the whole task.
fn spec(function: &str) -> CheckSpec {
    CheckSpec {
        typst: "check.typ".to_owned(),
        function: function.to_owned(),
        field: None,
    }
}

/// A `check` spec naming `function` and `field`.
fn field_spec(function: &str, field: &str) -> CheckSpec {
    CheckSpec {
        typst: "check.typ".to_owned(),
        function: function.to_owned(),
        field: Some(field.to_owned()),
    }
}

/// A bare project with `check.typ` at its root holding `source` — the fixture
/// every test here evaluates a rule against.
fn project(source: &str) -> tempfile::TempDir {
    let dir = tempdir();
    write_bare_project(dir.path());
    std::fs::write(dir.path().join("check.typ"), source).unwrap();
    dir
}

fn config() -> Config {
    parse_config("").unwrap()
}

/// The Problem's own rule: `x-goals` reaches a Typst function instead of the
/// filter language, which cannot name an `x-` field or a nested dict at all
/// (`truth/tasks/check-a-task-field-with-a-typst-function.typ`'s Problem
/// section). A task carrying the goal verdicts the task file's example gives
/// passes; the function only rejects a `neutral` verdict on a goal named
/// `readable-tasks`.
const CHECK_GOALS: &str = "\
#let check-goals(goals) = {
  if goals == none { return (\"x-goals is required\",) }
  let product = goals.at(\"product\", default: (:))
  if product.at(\"readable-tasks\", default: none) == \"neutral\" {
    (\"readable-tasks must not be neutral\",)
  } else {
    ()
  }
}
";

/// `x-goals: (product: (readable-tasks: <verdict>),)`, or a `product` table
/// with no `readable-tasks` entry at all when `verdict` is `None` — the
/// nested extras shape both `the_problem_config_...` and `a_pass_...` check
/// against, differing only in that one entry.
fn x_goals(verdict: Option<&str>) -> typst_harvest::HVal {
    use std::collections::BTreeMap;
    use typst_harvest::HVal;

    let mut product = BTreeMap::new();
    if let Some(verdict) = verdict {
        product.insert("readable-tasks".to_owned(), HVal::Str(verdict.to_owned()));
    }
    HVal::Dict(BTreeMap::from([(
        "product".to_owned(),
        HVal::Dict(product),
    )]))
}

/// A task carrying `x-goals` built by [`x_goals`].
fn task_with_goals(verdict: Option<&str>) -> Task {
    let mut task = bare_task("a");
    task.extras.insert("x-goals".to_owned(), x_goals(verdict));
    task
}

#[test]
fn the_problem_config_reaches_an_x_field_a_filter_expression_cannot_name() {
    let dir = project(CHECK_GOALS);
    let task = task_with_goals(Some("helps"));
    let check = CompiledCheck::new(&field_spec("check-goals", "x-goals"), dir.path(), &config());
    assert!(check.evaluate(&task, &config()).is_empty());
}

#[test]
fn a_pass_is_an_empty_array() {
    let dir = project(CHECK_GOALS);
    let task = task_with_goals(None);
    let check = CompiledCheck::new(&field_spec("check-goals", "x-goals"), dir.path(), &config());
    assert!(check.evaluate(&task, &config()).is_empty());
}

#[test]
fn two_breaches_are_two_strings() {
    let dir = project("#let check-two(task) = (\"first breach\", \"second breach\")\n");
    let task = bare_task("a");
    let check = CompiledCheck::new(&spec("check-two"), dir.path(), &config());
    let breaches = check.evaluate(&task, &config());
    assert_eq!(breaches, vec!["first breach", "second breach"]);
}

#[test]
fn a_panic_is_one_breach_carrying_its_message() {
    let dir = project("#let check-panic(task) = panic(\"boom\")\n");
    let task = bare_task("a");
    let check = CompiledCheck::new(&spec("check-panic"), dir.path(), &config());
    let breaches = check.evaluate(&task, &config());
    assert_eq!(breaches.len(), 1, "{breaches:?}");
    assert!(
        breaches.first().unwrap().contains("panicked with: boom"),
        "the panic's own message must survive: {breaches:?}"
    );
}

#[test]
fn a_missing_field_is_none() {
    let dir =
        project("#let check-none(value) = if value == none { (\"was none\",) } else { () }\n");
    let task = bare_task("a");
    let check = CompiledCheck::new(&field_spec("check-none", "x-absent"), dir.path(), &config());
    assert_eq!(check.evaluate(&task, &config()), vec!["was none"]);
}

#[test]
fn a_non_array_return_is_one_breach_naming_it() {
    let dir = project("#let check-bad(task) = \"not an array\"\n");
    let task = bare_task("a");
    let check = CompiledCheck::new(&spec("check-bad"), dir.path(), &config());
    let breaches = check.evaluate(&task, &config());
    assert_eq!(breaches.len(), 1, "{breaches:?}");
    assert!(
        breaches.first().unwrap().contains("not an array"),
        "must name what came back: {breaches:?}"
    );
}

/// A nested date in an `x-<ns>` slot survives as an [`HVal::Date`] and reaches
/// the function as a real Typst `datetime(...)` — the one place `check_literal`
/// diverges from `hval_to_typ`'s quoted-string round-trip.
#[test]
fn a_date_in_an_x_field_reaches_the_function_as_a_datetime() {
    let dir = project(
        "#let check-date(value) = if value == datetime(year: 2027, month: 1, day: 1) { () } else { (\"not a datetime\",) }\n",
    );
    let mut task = bare_task("a");
    task.extras.insert(
        "x-due".to_owned(),
        typst_harvest::HVal::Date("2027-01-01".to_owned()),
    );
    let check = CompiledCheck::new(&field_spec("check-date", "x-due"), dir.path(), &config());
    assert!(check.evaluate(&task, &config()).is_empty());
}

/// An [`HVal::Date`] that is not a `YYYY-MM-DD` falls back to a quoted string
/// rather than emitting a broken `datetime(...)` call.
#[test]
fn an_unparseable_date_in_an_x_field_falls_back_to_a_quoted_string() {
    let dir = project(
        "#let check-date(value) = if value == \"not-a-date\" { () } else { (\"unexpected: \" + repr(value),) }\n",
    );
    let mut task = bare_task("a");
    task.extras.insert(
        "x-due".to_owned(),
        typst_harvest::HVal::Date("not-a-date".to_owned()),
    );
    let check = CompiledCheck::new(&field_spec("check-date", "x-due"), dir.path(), &config());
    assert!(check.evaluate(&task, &config()).is_empty());
}

/// An array element that is not a string is its own literal repr, not an
/// error — a function returning the wrong element type still names it.
#[test]
fn a_non_string_array_element_is_rendered_by_its_literal() {
    let dir = project("#let check-mixed(task) = (\"first\", 42)\n");
    let task = bare_task("a");
    let check = CompiledCheck::new(&spec("check-mixed"), dir.path(), &config());
    assert_eq!(check.evaluate(&task, &config()), vec!["first", "42"]);
}

#[test]
fn an_unknown_function_is_one_config_error_at_load() {
    let dir = project("#let check-goals(goals) = ()\n");
    let toml = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.rules]]\ncheck = { typst = \"check.typ\", function = \"no-such-function\" }\n";
    let config = parse_config(toml).unwrap();
    let err = validate_load(&config, dir.path()).unwrap_err();
    assert!(
        err.contains("tasks.rules[0].check") && err.contains("no-such-function"),
        "{err}"
    );
}

#[test]
fn a_missing_typst_file_is_one_config_error_at_load() {
    let dir = tempdir();
    write_bare_project(dir.path());
    let toml = "[tasks]\nroots = [\"tasks\"]\n\n[[tasks.rules]]\ncheck = { typst = \"ghost.typ\", function = \"anything\" }\n";
    let config = parse_config(toml).unwrap();
    let err = validate_load(&config, dir.path()).unwrap_err();
    assert!(
        err.contains("tasks.rules[0].check") && err.contains("does not exist"),
        "{err}"
    );
}

#[test]
fn a_project_with_no_check_rule_passes_load_validation_untouched() {
    let dir = tempdir();
    write_bare_project(dir.path());
    let config = config();
    assert!(validate_load(&config, dir.path()).is_ok());
}
