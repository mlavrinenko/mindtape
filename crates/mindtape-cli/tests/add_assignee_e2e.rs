//! End-to-end tests for the env-backed `assignee` slot `add` splices in when
//! `[tasks.assignee] required` is set: `--describe` surfaces it, `--set` fills
//! it, `$MINDTAPE_PRINCIPAL` fills it silently, and an unset or out-of-list
//! principal fails loudly instead of writing an empty or wrong worker.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

mod support;

/// A config whose `[tasks.assignee]` is required with an allowed list: the
/// barebones template omits assignee, so `add` must splice in a choice slot
/// backed by `$MINDTAPE_PRINCIPAL`. The allowed names are declared actors, as
/// the actors registry requires.
const ASSIGNEE_REQUIRED_CONFIG: &str = "[tasks]\nroots = [\"tasks\"]\n\
     [tasks.assignee]\nallowed = [\"ada\", \"lin\"]\nrequired = true\n\
     [[actors]]\nname = \"ada\"\nkind = \"human\"\n\
     [[actors]]\nname = \"lin\"\nkind = \"human\"\n";

fn scaffold(dir: &Path, config: &str) {
    std::fs::create_dir_all(dir.join(".mindtape")).unwrap();
    std::fs::write(dir.join(".mindtape/config.toml"), config).unwrap();
    support::install_lib(dir);
}

fn describe_slot_names(dir: &Path) -> Vec<String> {
    let output = support::mt(dir)
        .args(["add", "--describe=json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    serde_json::from_slice::<serde_json::Value>(&output)
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|row| row.get("name").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

#[test]
fn injects_config_required_assignee_slot() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), ASSIGNEE_REQUIRED_CONFIG);

    // `--describe` surfaces the spliced-in assignee slot as a choice.
    let names = describe_slot_names(dir.path());
    assert!(names.iter().any(|n| n == "assignee"), "{names:?}");

    // Filling it writes a valid task.
    support::mt(dir.path())
        .args(["add", "--name", "with-assignee", "--set", "title=T"])
        .args(["--set", "assignee=ada"])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/with-assignee.typ")).unwrap();
    assert!(written.contains("assignee: \"ada\""), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    support::mt(dir.path()).arg("check").assert().success();

    // Omitting a config-required field is the existing missing-slot hard error.
    support::mt(dir.path())
        .args(["add", "--name", "no-assignee", "--set", "title=T"])
        .assert()
        .failure();
}

/// The required assignee splice is env-backed: a harness that sets
/// `$MINDTAPE_PRINCIPAL` files silently, with no `--set` and no prompt.
#[test]
fn principal_env_fills_required_assignee_silently() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), ASSIGNEE_REQUIRED_CONFIG);

    support::mt(dir.path())
        .env("MINDTAPE_PRINCIPAL", "ada")
        .args(["add", "--name", "env-ada", "--set", "title=T"])
        .assert()
        .success();
    let written = std::fs::read_to_string(dir.path().join("tasks/env-ada.typ")).unwrap();
    assert!(written.contains("assignee: \"ada\""), "{written}");
    assert!(!written.contains("slot("), "slots must be gone: {written}");

    support::mt(dir.path()).arg("check").assert().success();
}

/// An unset `$MINDTAPE_PRINCIPAL` is an unknown actor, not a default: a
/// required assignee with no other fill must fail loudly, never silently
/// write an empty worker.
#[test]
fn unset_principal_with_required_assignee_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), ASSIGNEE_REQUIRED_CONFIG);

    support::mt(dir.path())
        .env_remove("MINDTAPE_PRINCIPAL")
        .args(["add", "--name", "no-principal", "--set", "title=T"])
        .assert()
        .failure();
    assert!(!dir.path().join("tasks/no-principal.typ").exists());
}

/// `$MINDTAPE_PRINCIPAL` naming an actor outside the allowed list is a hard
/// error at fill time — the choice slot validates, so a wrong identity never
/// lands on a task.
#[test]
fn principal_outside_allowed_is_a_hard_error() {
    let dir = tempfile::tempdir().unwrap();
    scaffold(dir.path(), ASSIGNEE_REQUIRED_CONFIG);

    let output = support::mt(dir.path())
        .env("MINDTAPE_PRINCIPAL", "eve")
        .args(["add", "--name", "eve", "--set", "title=T"])
        .assert()
        .failure()
        .get_output()
        .stderr
        .clone();
    let stderr = String::from_utf8_lossy(&output);
    assert!(stderr.contains("eve"), "{stderr}");
    assert!(!dir.path().join("tasks/eve.typ").exists());
}
