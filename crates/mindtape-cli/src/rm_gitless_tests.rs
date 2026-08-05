#![allow(clippy::unwrap_used, clippy::indexing_slicing, unsafe_code)]

use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use mindtape::{RmStrategy, parse_config};

use super::super::{
    apply, plain_delete_plan, redact_git_error, referrers, reject_message, resolve_strategy,
    retire_apply, retire_plan, unlink_plan,
};
use super::{depends_survivor, gone_and_live, ids, survivor, task};
use crate::git;

/// A legend with `wip` → `done` (terminal), the pair most retire tests need.
fn legend() -> String {
    "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n"
        .to_owned()
}

/// A real task artifact for the file being retired — the status rewrite needs
/// a `task.with(…)` call to target.
fn artifact() -> String {
    "#import \"@local/mindtape:0.2.0\": *\n\
#show: task.with(\n  title: \"Gone\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n"
        .to_owned()
}

// --- resolve_strategy ---

#[test]
fn resolve_strategy_defaults_to_git_tombstone_in_a_repository() {
    let config = parse_config(&legend()).unwrap();
    assert_eq!(resolve_strategy(&config, true), RmStrategy::GitTombstone);
}

#[test]
fn resolve_strategy_defaults_to_reject_without_git() {
    let config = parse_config(&legend()).unwrap();
    assert_eq!(resolve_strategy(&config, false), RmStrategy::Reject);
}

#[test]
fn resolve_strategy_configured_value_wins_either_way() {
    let config = parse_config(&format!("{}\n[rm]\nstrategy = \"unlink\"\n", legend())).unwrap();
    assert_eq!(resolve_strategy(&config, true), RmStrategy::Unlink);
    assert_eq!(resolve_strategy(&config, false), RmStrategy::Unlink);
}

// --- reject ---

#[test]
fn reject_message_lists_every_survivor_and_id() {
    let mut referrers: BTreeMap<PathBuf, HashSet<String>> = BTreeMap::new();
    referrers.insert(PathBuf::from("tasks/live.typ"), ids(&["orphan", "gone"]));
    referrers.insert(PathBuf::from("tasks/other.typ"), ids(&["gone"]));

    let message = reject_message(&referrers);

    assert!(message.contains("cannot remove task(s): 3 reference(s)"));
    assert!(message.contains("tasks/live.typ"));
    assert!(
        message.contains("gone, orphan"),
        "sorted ids, got: {message}"
    );
    assert!(message.contains("tasks/other.typ"));
}

#[test]
fn referrers_maps_files_to_hits_and_skips_eligible() {
    // `live` references `gone`; `unrelated` does not; `a` (also being removed)
    // does but its edge is not waiting for attention.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("live.typ"), survivor("depends-on(\"gone.typ\")")).unwrap();
    std::fs::write(root.join("unrelated.typ"), "no links\n").unwrap();
    std::fs::write(root.join("a.typ"), survivor("related(\"gone.typ\")[why]")).unwrap();
    let tasks = vec![
        task("gone", "gone.typ", Some("done")),
        task("live", "live.typ", None),
        task("unrelated", "unrelated.typ", None),
        task("a", "a.typ", Some("done")),
    ];

    let map = referrers(&tasks, &ids(&["gone", "a"]), root).unwrap();

    assert_eq!(map.len(), 1);
    assert_eq!(map[&PathBuf::from("live.typ")], ids(&["gone"]));
}

#[test]
fn redact_git_error_hides_the_tasks_root() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let err = git::toplevel(root).unwrap_err();

    let message = redact_git_error(&err, root);

    assert!(
        !message.contains(&root.display().to_string()),
        "absolute path leaked: {message}"
    );
    assert!(message.contains("<tasks-root>"), "got: {message}");
}

// --- unlink ---

#[test]
fn unlink_plan_strips_edges_and_deletes() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), "gone\n").unwrap();
    std::fs::write(
        root.join("live.typ"),
        survivor("depends-on(\"gone.typ\") + related(\"gone.typ\")[why]"),
    )
    .unwrap();
    let tasks = gone_and_live("done");

    let plan = unlink_plan(&tasks, &ids(&["gone"]), root).unwrap();
    assert_eq!(plan.deletions.len(), 1);
    assert_eq!(plan.deletions[0].rel, PathBuf::from("gone.typ"));
    assert_eq!(plan.rewrites.len(), 1);
    assert_eq!(plan.rewrites[0].links, 2);
    assert!(!plan.rewrites[0].contents.contains("gone.typ"));

    apply(&plan).unwrap();
    assert!(!root.join("gone.typ").exists());
}

#[test]
fn unlink_plan_drops_whole_links_arg_when_value_fully_stripped() {
    // The only edge is the removed task's: the whole `links:` named argument
    // goes, separator comma included, leaving a bare task.with(...) — never a
    // dangling `links: ` key.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), "gone\n").unwrap();
    std::fs::write(root.join("live.typ"), survivor("depends-on(\"gone.typ\")")).unwrap();
    let tasks = gone_and_live("done");

    let plan = unlink_plan(&tasks, &ids(&["gone"]), root).unwrap();
    let contents = &plan.rewrites[0].contents;

    assert_eq!(plan.rewrites[0].links, 1);
    assert!(!contents.contains("links:"), "got: {contents}");
    assert!(contents.contains("#show: task.with("));
}

#[test]
fn unlink_plan_sweeps_plus_but_keeps_other_edges() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), "gone\n").unwrap();
    std::fs::write(
        root.join("live.typ"),
        survivor("depends-on(\"other.typ\") + related(\"gone.typ\")[why]"),
    )
    .unwrap();
    let tasks = gone_and_live("done");

    let plan = unlink_plan(&tasks, &ids(&["gone"]), root).unwrap();
    let contents = &plan.rewrites[0].contents;

    assert_eq!(plan.rewrites[0].links, 1);
    assert!(
        contents.contains("depends-on(\"other.typ\")"),
        "got: {contents}"
    );
    assert!(!contents.contains("gone.typ"), "got: {contents}");
    assert!(
        !contents.contains("+"),
        "dangling operator, got: {contents}"
    );
}

#[test]
fn unlink_plan_skips_survivors_without_edges() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), "gone\n").unwrap();
    std::fs::write(root.join("live.typ"), "no links here\n").unwrap();
    let tasks = gone_and_live("done");

    let plan = unlink_plan(&tasks, &ids(&["gone"]), root).unwrap();

    assert_eq!(plan.deletions.len(), 1);
    assert!(plan.rewrites.is_empty());
}

// --- retire ---

#[test]
fn retire_plan_defaults_to_first_terminal_status() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), artifact()).unwrap();
    std::fs::write(root.join("live.typ"), survivor("depends-on(\"gone.typ\")")).unwrap();
    let config = parse_config(&legend()).unwrap();
    let tasks = gone_and_live("wip");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();

    assert_eq!(plan.status, "done");
    assert!(plan.rename.is_none());
    assert_eq!(plan.deletions.len(), 0);
    assert_eq!(plan.rewrites.len(), 1);
    assert_eq!(plan.rewrites[0].abs, root.join("gone.typ"));
    assert!(
        plan.rewrites[0].contents.contains("done("),
        "no status flip"
    );
}

#[test]
fn retire_plan_uses_configured_status() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), artifact()).unwrap();
    let config = parse_config(
        "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n[retire]\nstatus = \"cancelled\"\n",
    )
    .unwrap();
    let tasks = gone_and_live("wip");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();

    assert_eq!(plan.status, "cancelled");
    assert!(plan.rewrites[0].contents.contains("cancelled("));
}

#[test]
fn retire_plan_errors_without_terminal_status() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), "gone\n").unwrap();
    let config = parse_config("[[tasks.status]]\nvalue = \"wip\"\n").unwrap();
    let tasks = gone_and_live("wip");

    let err = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap_err();

    let message = err.to_string();
    assert!(message.contains("no terminal status"), "got: {message}");
}

#[test]
fn retire_plan_rename_moves_file_into_archive_folder() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), artifact()).unwrap();
    let config = parse_config(&format!("{}\n[retire]\nrename = \".archive\"\n", legend())).unwrap();
    let tasks = gone_and_live("wip");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();
    assert_eq!(plan.deletions.len(), 1);
    assert_eq!(plan.deletions[0].rel, PathBuf::from("gone.typ"));
    assert_eq!(plan.rewrites.len(), 1);
    assert_eq!(plan.rewrites[0].rel, PathBuf::from(".archive/gone.typ"));

    retire_apply(&plan).unwrap();

    assert!(!root.join("gone.typ").exists());
    assert!(root.join(".archive/gone.typ").exists());
}

#[test]
fn retire_plan_rename_retargets_inbound_edges() {
    // A move breaks path-based resolution, so the survivor's edge to `gone`
    // must be rewritten onto the new relative path — the tombstone leaves no
    // dangling edge.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), artifact()).unwrap();
    std::fs::write(root.join("live.typ"), depends_survivor()).unwrap();
    let config = parse_config(&format!("{}\n[retire]\nrename = \".archive\"\n", legend())).unwrap();
    let tasks = gone_and_live("wip");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();
    assert_eq!(plan.link_rewrites.len(), 1);
    let rewritten = &plan.link_rewrites[0];
    assert_eq!(rewritten.rel, PathBuf::from("live.typ"));
    assert_eq!(rewritten.links, 1);
    assert!(
        rewritten
            .contents
            .contains("depends-on(\".archive/gone.typ\")"),
        "got: {}",
        rewritten.contents
    );

    retire_apply(&plan).unwrap();

    let live = std::fs::read_to_string(root.join("live.typ")).unwrap();
    assert!(
        live.contains("depends-on(\".archive/gone.typ\")"),
        "got: {live}"
    );
    assert!(root.join(".archive/gone.typ").exists());
}

#[test]
fn retire_plan_leaves_already_terminal_task_untouched() {
    // A tombstone already: the default status matches the current one, and
    // without a rename there is nothing to write at all.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), "gone\n").unwrap();
    let config = parse_config(&legend()).unwrap();
    let tasks = gone_and_live("done");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();

    assert!(plan.rewrites.is_empty());
    assert!(plan.deletions.is_empty());
}

#[test]
fn retire_plan_applies_without_git() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), artifact()).unwrap();
    let config = parse_config(&legend()).unwrap();
    let tasks = gone_and_live("wip");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();
    retire_apply(&plan).unwrap();

    let contents = std::fs::read_to_string(root.join("gone.typ")).unwrap();
    assert!(contents.contains("done("), "got: {contents}");
}

// --- batch awareness across strategies ---

#[test]
fn gitless_strategies_share_the_plain_delete_shape() {
    // The batch case: a removed task's only referrer is also being removed, so
    // no survivor references anything — the plain-delete plan applies and no
    // strategy (and no git) is needed.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("gone.typ"), "gone\n").unwrap();
    std::fs::write(root.join("a.typ"), survivor("depends-on(\"gone.typ\")")).unwrap();
    let tasks = vec![
        task("gone", "gone.typ", Some("done")),
        task("a", "a.typ", Some("done")),
    ];

    let map = referrers(&tasks, &ids(&["gone", "a"]), root).unwrap();
    assert!(map.is_empty());

    let plan = plain_delete_plan(&tasks, &ids(&["gone", "a"]), root);
    assert_eq!(plan.deletions.len(), 2);
    assert!(plan.rewrites.is_empty());
}

// The retire paths don't read survivors at all — only the eligible files — so
// their batch behaviour (eligible edges ignored) is the same as the single
// case exercised above.
