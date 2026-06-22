//! Tests for the `add` command.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::Path;

use mindtape::{Config, RawValue};

use super::{AddArgs, dest_dir, initial_status, inject_header, output_stem};
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
        template: None,
        values: vec![],
        set: vec![],
        into: ".".to_owned(),
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
    assert!(out.contains("status: status(\"proposed\","), "{out}");
}

#[test]
fn status_override_wins() {
    let out = inject_header(HEADER, &Config::default(), Some("wip")).unwrap();
    assert!(out.contains("status: status(\"wip\","), "{out}");
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
fn output_stem_errors_without_name_or_title() {
    assert!(output_stem(&bare_args(), &BTreeMap::new()).is_err());
}

#[test]
fn dest_default_is_first_root() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.tasks.roots = vec![mindtape::RootSpec::Short("truth/tasks".to_owned())];
    let project = Project {
        root: dir.path().to_path_buf(),
        config,
        outcome: mindtape::LoadOutcome::default(),
    };
    assert_eq!(
        dest_dir(".", &project).unwrap(),
        dir.path().join("truth/tasks")
    );
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
    assert!(
        written.contains("status: status(\"proposed\","),
        "{written}"
    );
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    // The filed task passes `check`.
    assert_cmd::Command::cargo_bin("mindtape")
        .unwrap()
        .current_dir(dir.path())
        .arg("check")
        .assert()
        .success();
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
