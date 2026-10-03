//! `mt check` prints every validation breach at `file:line` of the offending
//! field.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

const CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [[tasks.status]]\nvalue = \"wip\"\n\
     [[tasks.status]]\nvalue = \"done\"\nterminal = true\n";

fn check_text(source: &str) -> String {
    let dir = support::scaffold_project(CONFIG);
    std::fs::write(dir.path().join("tasks/setup-ci.typ"), source).unwrap();
    let out = support::mt(dir.path()).arg("check").output().unwrap();
    assert!(!out.status.success(), "the breach must fail the check");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_dangling_link_is_reported_at_its_links_line() {
    let text = check_text(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Set up CI\",\n  status: wip(2026, 6, 1),\n  \
         links: depends-on(\"nope.typ\"),\n)\n",
    );
    assert!(
        text.contains("error: tasks/setup-ci.typ:6: depends-on: link target"),
        "{text}"
    );
}

#[test]
fn a_cycle_is_reported_at_the_links_line_of_its_first_member() {
    let text = check_text(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Set up CI\",\n  status: wip(2026, 6, 1),\n  \
         links: depends-on(\"setup-ci.typ\"),\n)\n",
    );
    assert!(
        text.contains("error: tasks/setup-ci.typ:6: depends-on:"),
        "{text}"
    );
}
