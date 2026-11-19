//! End-to-end tests for the shape blocks and the `not allowed` label the bare
//! `mt explain` index opens on.
//!
//! `truth/tasks/2026-11/show-the-projects-shape-in-mt-explain.typ` and
//! `truth/tasks/2026-11/show-only-the-forbidden-gates-in-mt-explain.typ` are
//! the changes these pin. The listings keep `explain_e2e.rs`.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use crate::support;

use support::{json, mt, scaffold_project, stdout};

/// The bare index opens on the project's shape. It carries the roots with
/// their tags, the legend split by `terminal`, the `rm` filter, the scales,
/// the vocabularies and the commit trailer. `absent` names each key the config
/// leaves open.
#[test]
fn bare_opens_on_the_projects_shape() {
    let dir = scaffold_project("[tasks]\nroots = [\"tasks\"]\n");
    let text = stdout(dir.path(), &["explain"]);
    let expected = "project:
  roots: tasks (tag: tasks)
  statuses:
    open: proposed, accepted, wip
    terminal: done, cancelled, failed
  actors: none
task:
  priority: 0-9
  difficulty: 0-9
  estimate: any
  kinds: any
  links: depends-on, parent, recurred-from, supersedes, duplicates, related
  comments: allowed
  extensions: none
  templates: task (default: task)
commits:
  trailer: Refs
rm:
  filter: terminal
";
    assert!(text.starts_with(expected), "{text}");
}

/// A forbidden `[[on.rm]]` gate prints under `not allowed` in the block it
/// belongs to, and `mt explain rules --for rm` lists the hook it belongs to.
#[test]
fn a_forbidden_on_rm_gate_is_refused_and_listed_for_rm() {
    let dir = scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[on.rm]]\nname = \"clean\"\nwhen = 'terminal'\n\
         actions = [{ run = \"scripts/clean\", gate = true, message = \"clean first\" }]\n",
    );
    let text = stdout(dir.path(), &["explain"]);
    assert!(
        text.contains("rm:\n  filter: terminal\n  hooks (not allowed): clean\n"),
        "{text}"
    );

    let listed = stdout(dir.path(), &["explain", "rules", "--for", "rm"]);
    assert_eq!(listed, "error  on.rm.clean.actions[0]  clean first\n");
    let value = json(dir.path(), &["explain", "rules", "--for", "rm", "--json"]);
    assert_eq!(value["rules"][0]["section"], "on.rm");
}

/// A command the user has not allowed prints under `not allowed` in the block
/// it belongs to, and `mt allow` restores its severity. The JSON carries the
/// same label.
#[test]
fn a_command_the_user_has_not_allowed_prints_under_not_allowed() {
    let dir = scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[tasks.waits]]\nname = \"locked\"\nrun = \"scripts/locked {id}\"\n\n\
         [[on.flip]]\nwhen = 'into(\"done\")'\n\
         actions = [{ run = \"scripts/test\", gate = true }]\n",
    );
    let text = stdout(dir.path(), &["explain"]);
    for line in [
        "  waits (not allowed): scripts/locked {id}\n",
        "flip:\n  hooks (not allowed): scripts/test\n",
        "allow: mt allow records the approval for each command marked not allowed\n",
    ] {
        assert!(text.contains(line), "missing {line:?} in:\n{text}");
    }

    let allowed = mt(dir.path()).arg("allow").output().unwrap();
    assert!(allowed.status.success(), "{allowed:?}");
    let text = stdout(dir.path(), &["explain"]);
    assert!(text.contains("  waits: scripts/locked {id}\n"), "{text}");
    assert!(
        text.contains("flip:\n  hooks (error): scripts/test\n"),
        "{text}"
    );
    assert!(!text.contains("allow:"), "{text}");
    let value = json(dir.path(), &["explain", "--json"]);
    assert_eq!(
        value["rules"]["on.flip"],
        serde_json::json!({"error": ["scripts/test"]})
    );
}
