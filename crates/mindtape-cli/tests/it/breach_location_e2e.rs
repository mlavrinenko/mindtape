//! `mt check` prints every validation breach at `file:line` of the offending
//! field.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;
use mindtape::test_support::write_file;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

fn check_text(source: &str) -> String {
    let dir = support::scaffold_project(CONFIG);
    write_file(dir.path(), "tasks/setup-ci.typ", source);
    let out = support::mt(dir.path()).arg("check").output().unwrap();
    assert!(!out.status.success(), "the breach must fail the check");
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn a_dangling_link_is_reported_at_its_links_line() {
    let text = check_text(mindtape::typst_task!(
        "(\n  title: \"Set up CI\",\n  status: wip(2026, 6, 1),\n  \
         links: depends-on(\"nope.typ\"),\n)\n"
    ));
    assert!(
        text.contains("error: tasks/setup-ci.typ:6: depends-on: link target"),
        "{text}"
    );
}

#[test]
fn a_cycle_is_reported_at_the_links_line_of_its_first_member() {
    let text = check_text(mindtape::typst_task!(
        "(\n  title: \"Set up CI\",\n  status: wip(2026, 6, 1),\n  \
         links: depends-on(\"setup-ci.typ\"),\n)\n"
    ));
    assert!(
        text.contains("error: tasks/setup-ci.typ:6: depends-on:"),
        "{text}"
    );
}

/// The stderr of `mt check` over a task file that fails to evaluate.
fn eval_warning(source: &str) -> String {
    let dir = support::scaffold_project(CONFIG);
    write_file(dir.path(), "tasks/setup-ci.typ", source);
    let out = support::mt(dir.path()).arg("check").output().unwrap();
    assert!(!out.status.success(), "the load error must fail the check");
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn a_syntax_error_is_reported_at_its_line() {
    let text = eval_warning(mindtape::typst_task!(
        "(\n  status: wip(2026, 6, 1),\n  \
         title: \"Set up CI\"\n  assignee: \"ada\",\n)\n"
    ));
    assert!(
        text.contains("error: tasks/setup-ci.typ:5:21: expected comma"),
        "{text}"
    );
}

#[test]
fn a_misspelled_field_is_reported_at_its_line() {
    let text = eval_warning(mindtape::typst_task!(
        "(\n  title: \"Set up CI\",\n  status: wip(2026, 6, 1),\n  \
         titel: \"typo\",\n)\n"
    ));
    assert!(
        text.contains("error: tasks/setup-ci.typ:6:3: unexpected argument: titel"),
        "{text}"
    );
}

/// A misspelled status helper gets the statuses hint, and no advice about `#`.
#[test]
fn an_unknown_status_helper_points_at_the_statuses() {
    let text = eval_warning(mindtape::typst_task!(
        "(\n  title: \"Set up CI\",\n  status: finished(2026, 10, 19),\n)\n"
    ));
    assert!(
        text.contains(
            "error: tasks/setup-ci.typ:5:11: unknown variable: finished\n\
             hint: run mt flip --list to see the statuses\n"
        ),
        "{text}"
    );
    assert!(!text.contains("a literal #"), "{text}");
}

/// Any other unknown name in the header, such as a misspelled framework
/// helper, gets neither hint.
#[test]
fn an_unknown_helper_elsewhere_in_the_header_gets_no_hint() {
    let text = eval_warning(mindtape::typst_task!(
        "(\n  title: \"Set up CI\",\n  \
         priority: icee(impact: 8, confidence: 0.7, ease: 6),\n)\n"
    ));
    assert!(
        text.contains("error: tasks/setup-ci.typ:5:13: unknown variable: icee\n"),
        "{text}"
    );
    assert!(!text.contains("hint:"), "{text}");
}

#[test]
fn an_unknown_variable_in_the_body_gets_the_escape_hint() {
    let text = eval_warning(mindtape::typst_task!(
        "(title: \"Set up CI\")\nChannel #N carries it.\n"
    ));
    assert!(
        text.contains(
            "error: tasks/setup-ci.typ:4:10: unknown variable: N\n\
             hint: a literal # in a task body opens a Typst expression: \
             write \\# or put the prose in a code block\n"
        ),
        "{text}"
    );
}

/// The stderr of `mt ls` over a task file, which loads or does not.
fn ls_stderr(source: &str) -> String {
    let dir = support::scaffold_project(CONFIG);
    write_file(dir.path(), "tasks/setup-ci.typ", source);
    let out = support::mt(dir.path()).arg("ls").output().unwrap();
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn a_load_error_in_a_listing_is_a_warning_with_typsts_hint() {
    let text = ls_stderr(mindtape::typst_task!(
        "(title: \"Set up CI\")\n\
         #let a = 1\n#a-b\n"
    ));
    assert!(
        text.contains("warning: tasks/setup-ci.typ:5:2: unknown variable: a-b\nhint: "),
        "{text}"
    );
}

#[test]
fn a_hash_in_a_body_gets_the_escape_hint_once() {
    let text = ls_stderr(mindtape::typst_task!(
        "(title: \"Set up CI\")\n\
         Channel #N carries it.\n"
    ));
    assert!(text.contains("unknown variable: N"), "{text}");
    assert_eq!(
        text.matches("a literal # in a task body").count(),
        1,
        "{text}"
    );
}

#[test]
fn a_typst_warning_on_a_file_that_loads_is_its_own_warning_line() {
    let text = ls_stderr(
        "<orphan>\n\
         #import \"@local/mindtape:0.2.0\": *\n\
         #show: task.with(title: \"Set up CI\")\n",
    );
    assert!(
        text.contains(
            "warning: tasks/setup-ci.typ:1:1: label `<orphan>` is not attached to anything"
        ),
        "{text}"
    );
}
