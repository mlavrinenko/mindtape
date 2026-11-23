#![allow(clippy::unwrap_used)]

use super::{CompiledCheck, add_argument, validate_load};
use crate::config::Config;
use crate::model::Task;
use crate::parse_config;
use crate::rules::CheckSpec;
use crate::test_support::tempdir;
use crate::test_support::write_file;
use crate::test_support::{bare_task, body_marker, write_bare_project};
use std::path::Path;
use typst_harvest::HVal;

/// The value an add `derive` reads never carries the draft's synthetic
/// identity, place or folder-derived tags.
#[test]
fn an_add_argument_drops_the_identity_and_derived_values() {
    let config = parse_config("").unwrap();
    let task = Task {
        id: "draft-id".to_owned(),
        file: "tasks/__draft__.typ".into(),
        derived_tags: vec!["sandbox".to_owned()],
        ..Task::default()
    };
    let HVal::Dict(draft) = add_argument(&task, &config, "task") else {
        panic!("a draft is a dict");
    };
    for key in ["id", "file", "derived_tags", "derived"] {
        assert!(!draft.contains_key(key), "{key} must be dropped");
    }
    assert_eq!(draft.get("template"), Some(&HVal::Str("task".to_owned())));
}

/// A `check` spec naming `function` in `check.typ` at the project root, with
/// no `field`. The function then receives the whole task.
fn spec(function: &str) -> CheckSpec {
    CheckSpec {
        typst: "check.typ".to_owned(),
        function: function.to_owned(),
        field: None,
        root: None,
        resolved: None,
    }
}

/// A `check` spec naming `function` and `field`.
fn field_spec(function: &str, field: &str) -> CheckSpec {
    CheckSpec {
        typst: "check.typ".to_owned(),
        function: function.to_owned(),
        field: Some(field.to_owned()),
        root: None,
        resolved: None,
    }
}

/// A bare project with `check.typ` at its root holding `source`, the fixture
/// every test here evaluates a rule against.
fn project(source: &str) -> tempfile::TempDir {
    let dir = tempdir();
    write_bare_project(dir.path());
    write_file(dir.path(), "check.typ", source);
    dir
}

fn config() -> Config {
    parse_config("").unwrap()
}

/// The Problem's own rule: `x-goals` reaches a Typst function instead of the
/// filter language, which cannot reference an `x-` field or a nested dict at
/// all (`truth/tasks/2026-09/check-a-task-field-with-a-typst-function.typ`'s
/// Problem section). A task carrying the goal verdicts the task file's example
/// gives passes. The function only rejects a `neutral` verdict on a goal named
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
/// with no `readable-tasks` entry at all when `verdict` is `None`. That is the
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
/// the function as a real Typst `datetime(...)` through `HVal::to_typst`. There
/// it diverges from `hval_to_typ`'s quoted-string round-trip.
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

/// An infinite float in an `x-<ns>` slot reaches the function as Typst's own
/// `float.inf`, not Rust's `inf`, which Typst reads as an unknown variable.
#[test]
fn an_infinite_float_in_an_x_field_reaches_the_function() {
    let dir = project(
        "#let check-inf(value) = if value == float.inf { () } else { (\"unexpected: \" + repr(value),) }\n",
    );
    let mut task = bare_task("a");
    task.extras.insert(
        "x-cap".to_owned(),
        typst_harvest::HVal::Float(f64::INFINITY),
    );
    let check = CompiledCheck::new(&field_spec("check-inf", "x-cap"), dir.path(), &config());
    assert_eq!(check.evaluate(&task, &config()), Vec::<String>::new());
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
/// error. A function returning the wrong element type still names it.
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

/// The reserved `markers` field: the body's metadata markers as `(label, value)`
/// dicts in document order, `label` `none` for an unlabelled marker. It is the
/// one check input that is neither a canonical field nor an `x-<ns>` slot, and
/// the goals DSL reads it.
const CHECK_MARKERS: &str = "\
#let check-markers(markers) = {
  if markers == none { return (\"markers must be an array, not none\",) }
  if markers.len() != 2 { return (\"two markers expected: \" + repr(markers),) }
  let first = markers.at(0)
  if first.label != \"goals\" { return (\"first label: \" + repr(first.label),) }
  if first.value != (goal: \"implementable-tasks\", verdict: \"helps\") {
    return (\"first value: \" + repr(first.value),)
  }
  let second = markers.at(1)
  if second.label != none { return (\"second label: \" + repr(second.label),) }
  if second.value != \"bare\" { return (\"second value: \" + repr(second.value),) }
  ()
}
";

/// A task carrying a labelled marker and an unlabelled one, in that order.
fn task_with_body_markers() -> Task {
    use std::collections::BTreeMap;

    let mut task = bare_task("a");
    task.markers = vec![
        body_marker(
            Some("goals"),
            HVal::Dict(BTreeMap::from([
                (
                    "goal".to_owned(),
                    HVal::Str("implementable-tasks".to_owned()),
                ),
                ("verdict".to_owned(), HVal::Str("helps".to_owned())),
            ])),
        ),
        body_marker(None, HVal::Str("bare".to_owned())),
    ];
    task
}

/// The breaches a rule calling `function` over `field` reports for `task`,
/// evaluated in the project at `dir`.
fn breaches(dir: &Path, function: &str, field: &str, task: &Task) -> Vec<String> {
    let config = config();
    let spec = field_spec(function, field);
    CompiledCheck::new(&spec, dir, &config).evaluate(task, &config)
}

#[test]
fn the_reserved_markers_field_reaches_the_function_as_dicts() {
    let dir = project(CHECK_MARKERS);
    let task = task_with_body_markers();
    let found = breaches(dir.path(), "check-markers", "markers", &task);
    assert!(found.is_empty(), "{found:?}");
}

/// A task whose body carries an empty marker list hands the function an empty
/// array, not `none`. A check can then call `.len()` on it without a nil guard.
const CHECK_NO_MARKERS: &str = "\
#let check-no-markers(markers) = {
  if markers == none { return (\"markers must be an array, not none\",) }
  if markers.len() != 0 { return (\"expected no markers: \" + repr(markers),) }
  ()
}
";

#[test]
fn an_absent_markers_field_is_an_empty_array() {
    let dir = project(CHECK_NO_MARKERS);
    let task = bare_task("a");
    let found = breaches(dir.path(), "check-no-markers", "markers", &task);
    assert!(found.is_empty(), "{found:?}");
}
