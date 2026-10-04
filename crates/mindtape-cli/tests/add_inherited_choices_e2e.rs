//! End-to-end tests for a template's own slot that is the whole value of a
//! `task.with` field with config `values`: it is a choice over those values,
//! in `mt add`, `--describe` and `mt check`'s template pass alike.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

mod support;

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
        stderr.contains(r#"slot "kind" value "chore" is not one of: feat, test"#),
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
