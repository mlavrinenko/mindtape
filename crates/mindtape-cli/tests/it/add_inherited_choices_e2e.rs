//! End-to-end tests for a template's own slot that is the whole value of a
//! `task.with` field with config `values`: it is a choice over those values,
//! in `mt add`, `--describe` and `mt check`'s template pass alike.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support;

/// `kind` lists its values in an order whose first value is not the
/// template's example.
const KIND_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.kind]\nvalues = [\"feat\", \"test\", \"chore\"]\nrequired = true\n";

/// A config whose `kind` refuses `chore`, so a fill of it has a list to fail.
const NO_CHORE_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.kind]\nvalues = [\"feat\", \"test\"]\nrequired = true\n";

/// A template that declares its own `kind` slot with an example and no
/// `choices`.
const OWN_KIND_TEMPLATE: &str = "#import \"@local/mindtape:0.2.0\": *\n\
     #show: task.with(\n  title: slot(\"title\", example: \"T\"),\n  \
     kind: slot(\"kind\", example: \"test\"),\n)\n";

#[test]
fn a_templates_own_slot_refuses_a_value_outside_config_naming_the_choices() {
    let dir = support::project_with_template_and_config(NO_CHORE_CONFIG, OWN_KIND_TEMPLATE);
    let output = support::mt(dir.path())
        .args(["add", "--name", "x", "--set", "title=x", "--", "kind=chore"])
        .assert()
        .failure()
        .get_output()
        .clone();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: not one of the slot's choices: 'chore' for slot 'kind': expected one of feat, test"),
        "{stderr}"
    );
    assert!(!dir.path().join("tasks/x.typ").exists());
}

#[test]
fn describe_lists_the_inherited_choices() {
    let dir = support::project_with_template_and_config(KIND_CONFIG, OWN_KIND_TEMPLATE);
    let slots = support::describe_slots(dir.path());
    let kind = slots
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "kind")
        .unwrap();
    assert_eq!(kind["kind"], "choice", "{kind}");
    assert_eq!(
        kind["choices"],
        serde_json::json!(["feat", "test", "chore"])
    );
    assert_eq!(kind["example"], "test", "{kind}");
}

#[test]
fn an_inherited_choice_files_a_listed_value() {
    let dir = support::project_with_template_and_config(KIND_CONFIG, OWN_KIND_TEMPLATE);
    support::mt(dir.path())
        .args(["add", "--name", "x", "--set", "title=x", "--", "kind=chore"])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/x.typ")).unwrap();
    assert!(written.contains("kind: \"chore\""), "{written}");
    assert!(!written.contains("choices"), "{written}");
    support::mt(dir.path()).arg("check").assert().success();
}

/// The template check fills the template's own example, not the config's
/// first value.
#[test]
fn the_template_check_files_the_slots_own_example() {
    const DOCS_ONLY_TEST: &str = "[tasks]\nroots = [\"tasks\"]\n\
         [tasks.fields.kind]\nvalues = [\"feat\", \"test\"]\nrequired = true\n\
         [[tasks.rules]]\nname = \"no-feat\"\nassert = \"kind != \\\"feat\\\"\"\n\
         severity = \"error\"\n";
    let dir = support::project_with_template_and_config(DOCS_ONLY_TEST, OWN_KIND_TEMPLATE);
    support::mt(dir.path()).arg("check").assert().success();
}

/// A list-valued field: `x-machine` takes any of the listed machines.
const MACHINE_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.fields.x-machine]\nvalues = [\"alpha\", \"beta\"]\nmultiple = true\n\
     required = true\nexample = [\"alpha\"]\n";

/// A template that declares its own `x-machine` slot, with no `choices`.
const OWN_MACHINE_TEMPLATE: &str = "#import \"@local/mindtape:0.2.0\": *\n\
     #show: task.with(\n  title: slot(\"title\", example: \"T\"),\n  \
     x-machine: slot(\"machine\", example: (\"alpha\",)),\n)\n";

/// A template that never mentions the field, so `mt add` splices its slot.
const PLAIN_TEMPLATE: &str = "#import \"@local/mindtape:0.2.0\": *\n\
     #show: task.with(\n  title: slot(\"title\", example: \"T\"),\n)\n";

fn file_machines(template: &str, slot: &str) -> String {
    let dir = support::project_with_template_and_config(MACHINE_CONFIG, template);
    support::mt(dir.path())
        .args(["add", "--name", "x", "--set", "title=x", "--"])
        .args([format!("{slot}+=alpha"), format!("{slot}+=beta")])
        .assert()
        .success();
    support::mt(dir.path()).arg("check").assert().success();
    std::fs::read_to_string(dir.path().join("tasks/x.typ")).unwrap()
}

#[test]
fn a_spliced_multiple_field_files_a_list() {
    let written = file_machines(PLAIN_TEMPLATE, "x-machine");
    assert!(
        written.contains(r#"x-machine: ("alpha", "beta")"#),
        "{written}"
    );
}

#[test]
fn an_inherited_multiple_field_files_a_list() {
    let written = file_machines(OWN_MACHINE_TEMPLATE, "machine");
    assert!(
        written.contains(r#"x-machine: ("alpha", "beta")"#),
        "{written}"
    );
}

#[test]
fn a_multiple_field_is_described_as_a_multi_choice() {
    let dir = support::project_with_template_and_config(MACHINE_CONFIG, PLAIN_TEMPLATE);
    let slots = support::describe_slots(dir.path());
    let machine = slots
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "x-machine")
        .unwrap();
    assert_eq!(machine["kind"], "multichoice", "{machine}");
    assert_eq!(machine["choices"], serde_json::json!(["alpha", "beta"]));
}
