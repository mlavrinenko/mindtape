//! End-to-end tests for `[add] verify-when` and the `--verify` / `--no-verify`
//! overrides. A run that skips the check files an incomplete task, runs no
//! `[[on.add]]` hook, and ends on a note that points at `mt check`.
//!
//! They live apart from `add_e2e.rs` along that subject seam, and here rather
//! than as inline unit tests because `cargo_bin` needs the compiled binary.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use crate::support;

/// A template whose `as-is` slot carries neither a default nor a value, so a
/// run that skips the check has something to leave behind.
const REQUIRED_SLOT_TEMPLATE: &str = mindtape::typst_task!(
    r#"(title: slot("title", example: "Fix the login redirect"))

== AS-IS

#slot("as-is", kind: "raw", example: ```typ
+ Run `mt ls`.
```)
"#
);

/// A project whose `[add] verify-when` is `false`, which skips the check on
/// every run. `hook` appends an `[[on.add]]` entry that sets `tags`.
fn project(hook: bool) -> tempfile::TempDir {
    let mut config = "[tasks]\nroots = [\"tasks\"]\n\n[add]\nverify-when = \"false\"\n".to_owned();
    if hook {
        config.push_str(
            "\n[[on.add]]\nname = \"tag-it\"\nactions = [{ set = { tags = \"hooked\" } }]\n",
        );
    }
    support::project_with_template_and_config(&config, REQUIRED_SLOT_TEMPLATE)
}

/// The task `mt add` filed in `dir`, as its source.
fn filed(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("tasks/incomplete.typ")).unwrap()
}

/// `[add] verify-when = "false"` files the task with its required slot left as
/// the `slot(...)` call, and ends on a note that points at `mt check`. The run
/// has no terminal, so `!tty()` would verify. `false` is the policy under test.
#[test]
fn a_skipped_verification_files_the_slot_call_and_names_check() {
    let dir = project(false);

    let done = support::ok(
        dir.path(),
        &["add", "--name", "incomplete", "--set", "title=Incomplete"],
    );
    let stderr = support::stderr_of(&done);

    assert!(
        filed(dir.path()).contains("slot(\"as-is\""),
        "{}",
        filed(dir.path())
    );
    assert!(
        stderr.contains("note: filed without verification; `mt check` lists what is missing"),
        "{stderr}"
    );
}

/// `--verify` demands the check whatever `verify-when` says, so the same
/// project that files an incomplete task refuses one on request.
#[test]
fn verify_overrides_a_policy_that_skips_the_check() {
    let dir = project(false);

    let done = support::assert_fails(
        dir.path(),
        &[
            "add",
            "--verify",
            "--name",
            "incomplete",
            "--set",
            "title=Incomplete",
        ],
    );
    let stderr = support::stderr_of(&done);

    assert!(
        stderr.contains("missing a required slot: 'as-is'"),
        "{stderr}"
    );
    assert!(!dir.path().join("tasks/incomplete.typ").exists());
}

/// A run that skips the check doesn't run an `[[on.add]]` hook. A hook reads
/// fields the fill has not written.
#[test]
fn a_skipped_verification_runs_no_add_hook() {
    let dir = project(true);

    support::run_ok(
        dir.path(),
        &["add", "--name", "incomplete", "--set", "title=Incomplete"],
    );

    assert!(
        !filed(dir.path()).contains("hooked"),
        "{}",
        filed(dir.path())
    );
}
