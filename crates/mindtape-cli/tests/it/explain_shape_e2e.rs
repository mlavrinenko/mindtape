//! End-to-end tests for the shape blocks and the `refusals:` block the bare
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
  rm: terminal only
task:
  priority: 0-9
  difficulty: 0-9
  links: depends-on, parent, recurred-from, supersedes, duplicates, related
  templates: task (default: task)
commits:
  trailer: Refs
absent: actors, estimate, kinds, comments, extensions
rules: none
";
    assert!(text.starts_with(expected), "{text}");
}

/// A forbidden `[[on.rm]]` gate prints in `refusals:` with the section that
/// declares it, and `mt explain rules --for rm` lists the hook it belongs to.
#[test]
fn a_forbidden_on_rm_gate_is_refused_and_listed_for_rm() {
    let dir = scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[on.rm]]\nname = \"clean\"\nwhen = 'terminal'\n\
         actions = [{ run = \"scripts/clean\", gate = true, message = \"clean first\" }]\n",
    );
    let text = stdout(dir.path(), &["explain"]);
    assert!(
        text.contains("  on.rm: scripts/clean (not allowed; ask the user to run `mt allow`)\n"),
        "{text}"
    );
    assert!(text.contains("  on.rm: clean\n"), "{text}");

    let listed = stdout(dir.path(), &["explain", "rules", "--for", "rm"]);
    assert_eq!(listed, "error  on.rm.clean.actions[0]  clean first\n");
    let value = json(dir.path(), &["explain", "rules", "--for", "rm", "--json"]);
    assert_eq!(value["rules"][0]["section"], "on.rm");
}

/// A gate the user has not allowed prints in `refusals:` with the section that
/// declares it, and `mt allow` clears the block. The JSON carries the same
/// list.
#[test]
fn refusals_lists_only_the_gates_the_user_has_not_allowed() {
    let dir = scaffold_project(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[tasks.waits]]\nname = \"locked\"\nrun = \"scripts/locked {id}\"\n\n\
         [[on.flip]]\nwhen = 'into(\"done\")'\n\
         actions = [{ run = \"scripts/test\", gate = true }]\n",
    );
    let text = stdout(dir.path(), &["explain"]);
    let expected = "refusals:
  waits: scripts/locked {id} (not allowed; ask the user to run `mt allow`)
  on.flip: scripts/test (not allowed; ask the user to run `mt allow`)
";
    assert!(text.contains(expected), "{text}");

    let allowed = mt(dir.path()).arg("allow").output().unwrap();
    assert!(allowed.status.success(), "{allowed:?}");
    let text = stdout(dir.path(), &["explain"]);
    assert!(!text.contains("refusals:"), "{text}");
    let value = json(dir.path(), &["explain", "--json"]);
    assert_eq!(value["refusals"], serde_json::json!([]));
}
