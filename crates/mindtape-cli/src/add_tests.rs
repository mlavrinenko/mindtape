//! Tests for the `add` command.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::Path;

use mindtape::{Config, RawValue};

use super::{AddArgs, dest_dir, fill_title_from_args, initial_status, inject_header, output_stem};
use crate::util::Project;

const PRELUDE: &str = include_str!("../../../lib/prelude.typ");

const HEADER: &str = r#"#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Do thing",
)

== Summary
"#;

fn bare_args() -> AddArgs {
    AddArgs {
        title: vec![],
        template: None,
        values: vec![],
        set: vec![],
        into: ".".to_owned(),
        to: None,
        name: None,
        status: None,
        edit: false,
        describe: None,
        list: false,
        print: false,
        no_verify: false,
    }
}

#[test]
fn injects_default_status_proposed() {
    let out = inject_header(HEADER, &Config::default(), None).unwrap();
    assert!(out.contains("status: proposed("), "{out}");
    assert!(!out.contains("status(\"proposed\""), "{out}");
}

#[test]
fn status_override_wins() {
    let out = inject_header(HEADER, &Config::default(), Some("wip")).unwrap();
    assert!(out.contains("status: wip("), "{out}");
}

#[test]
fn custom_status_falls_back_to_verbose_form() {
    // A name the prelude ships no shortcut for keeps the `status("…", …)` form.
    let out = inject_header(HEADER, &Config::default(), Some("icebox")).unwrap();
    assert!(out.contains("status: status(\"icebox\","), "{out}");
}

#[test]
fn initial_status_is_first_non_terminal() {
    let config: Config = mindtape::parse_config(
        "[[tasks.status]]\nvalue = \"done\"\nterminal = true\n\
         [[tasks.status]]\nvalue = \"open\"\n",
    )
    .unwrap();
    assert_eq!(initial_status(&config).as_deref(), Some("open"));
}

#[test]
fn output_stem_prefers_name_then_title_slug() {
    let mut args = bare_args();
    args.name = Some("explicit".to_owned());
    assert_eq!(output_stem(&args, &BTreeMap::new()).unwrap(), "explicit");

    let args = bare_args();
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("My New Task".to_owned()));
    assert_eq!(output_stem(&args, &values).unwrap(), "my-new-task");
}

#[test]
fn title_arg_joins_words_when_unset() {
    let mut args = bare_args();
    args.title = vec!["cleanup".to_owned(), "kitchen".to_owned()];
    let mut values = BTreeMap::new();
    fill_title_from_args(&mut values, &args);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("cleanup kitchen".to_owned()))
    );
}

#[test]
fn title_arg_yields_to_higher_precedence_title() {
    let mut args = bare_args();
    args.title = vec!["positional".to_owned()];
    let mut values = BTreeMap::new();
    values.insert("title".to_owned(), RawValue::Str("explicit".to_owned()));
    fill_title_from_args(&mut values, &args);
    assert_eq!(
        values.get("title"),
        Some(&RawValue::Str("explicit".to_owned()))
    );
}

#[test]
fn output_stem_errors_without_name_or_title() {
    assert!(output_stem(&bare_args(), &BTreeMap::new()).is_err());
}

/// A `Project` rooted at `dir` whose single configured root is `root`.
fn project_with_root(dir: &Path, root: &str) -> Project {
    let mut config = Config::default();
    config.tasks.roots = vec![mindtape::RootSpec::Short(root.to_owned())];
    Project {
        root: dir.to_path_buf(),
        config,
        outcome: mindtape::LoadOutcome::default(),
    }
}

#[test]
fn dest_default_is_first_root() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_root(dir.path(), "truth/tasks");
    assert_eq!(
        dest_dir(".", None, &project).unwrap(),
        dir.path().join("truth/tasks")
    );
}

#[test]
fn dest_to_nests_subfolder_under_root() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_root(dir.path(), "truth/tasks");
    assert_eq!(
        dest_dir(".", Some("shopping"), &project).unwrap(),
        dir.path().join("truth/tasks/shopping")
    );
    // Nested subpaths are allowed.
    assert_eq!(
        dest_dir(".", Some("shopping/weekly"), &project).unwrap(),
        dir.path().join("truth/tasks/shopping/weekly")
    );
}

#[test]
fn dest_to_rejects_absolute_subfolder() {
    let dir = tempfile::tempdir().unwrap();
    let project = project_with_root(dir.path(), "tasks");
    assert!(dest_dir(".", Some("/etc"), &project).is_err());
}

/// Scaffold a temp project that can evaluate `@local/mindtape` via the in-tree
/// lib override the loader honours.
fn scaffold(dir: &Path) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(
        dir.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(dir.join("lib/prelude.typ"), PRELUDE).unwrap();
    std::fs::write(
        dir.join("lib/typst.toml"),
        "[package]\nname = \"mindtape\"\nversion = \"0.2.0\"\nentrypoint = \"prelude.typ\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("tasks")).unwrap();
}

#[test]
fn end_to_end_files_and_validates_a_task() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    assert_cmd::Command::cargo_bin("mindtape")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "wire-add", "--set", "title=Wire add"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/wire-add.typ")).unwrap();
    assert!(written.contains("title: \"Wire add\""), "{written}");
    assert!(written.contains("status: proposed("), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task passes `check`.
    assert_cmd::Command::cargo_bin("mindtape")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success();
}

#[test]
fn end_to_end_files_from_positional_title() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    assert_cmd::Command::cargo_bin("mindtape")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "cleanup", "kitchen"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/cleanup-kitchen.typ")).unwrap();
    assert!(written.contains("title: \"cleanup kitchen\""), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");
}

#[test]
fn end_to_end_nests_into_subfolder() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());

    assert_cmd::Command::cargo_bin("mindtape")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--to", "shopping", "Potatoes"])
        .assert()
        .success();

    let written = std::fs::read_to_string(dir.path().join("tasks/shopping/potatoes.typ")).unwrap();
    assert!(written.contains("title: \"Potatoes\""), "{written}");
}

/// Slot names from `add --describe=json` run in `dir`.
fn describe_slot_names(dir: &Path) -> Vec<String> {
    let output = assert_cmd::Command::cargo_bin("mindtape")
        .unwrap()
        .current_dir(dir)
        .args(["add", "--describe=json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).unwrap();
    parsed
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn describe_json_emits_slot_schema() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    assert!(describe_slot_names(dir.path()).iter().any(|n| n == "title"));
}

/// A config whose `[tasks.priority]` is required: the barebones template omits
/// priority, so `add` must splice it in.
const PRIORITY_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.priority]\nkind = \"int\"\nmin = 0\nmax = 9\nrequired = true\n";

#[test]
fn injects_config_required_priority_slot() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    std::fs::write(
        dir.path().join(".mindtape/config.toml"),
        PRIORITY_REQUIRED_CONFIG,
    )
    .unwrap();

    // `--describe` surfaces the spliced-in priority slot.
    let names = describe_slot_names(dir.path());
    assert!(names.iter().any(|n| n == "priority"), "{names:?}");

    // Filling it writes a valid task.
    assert_cmd::Command::cargo_bin("mindtape")
        .unwrap()
        .current_dir(dir.path())
        .args([
            "add",
            "--name",
            "with-pri",
            "--set",
            "title=T",
            "--set",
            "priority=7",
        ])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-pri.typ")).unwrap();
    assert!(written.contains("priority: 7"), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // Omitting a config-required field is the existing missing-slot hard error.
    assert_cmd::Command::cargo_bin("mindtape")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "no-pri", "--set", "title=T"])
        .assert()
        .failure();
}

#[test]
fn collision_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path());
    std::fs::write(dir.path().join("tasks/dup.typ"), "existing").unwrap();
    assert_cmd::Command::cargo_bin("mindtape")
        .unwrap()
        .current_dir(dir.path())
        .args(["add", "--name", "dup", "--set", "title=Dup"])
        .assert()
        .failure();
}
