#![allow(clippy::unwrap_used, clippy::indexing_slicing, unsafe_code)]

use mindtape::test_support::{tempdir, tempdir_with_files};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use mindtape::{RmStrategy, parse_config};

use super::super::{
    apply, plain_delete_plan, redact_git_error, referrers, reject_diag, resolve_strategy,
    unlink_plan,
};
use super::{gone_and_live, ids, survivor, task};
use crate::git;

/// A legend with `wip` → `done` (terminal), the pair most retire tests need.
pub(super) fn legend() -> String {
    "[[tasks.status]]\nvalue = \"wip\"\n[[tasks.status]]\nvalue = \"done\"\nterminal = true\n"
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
fn the_reject_refusal_names_every_survivor_and_its_ids() {
    let mut referrers: BTreeMap<PathBuf, HashSet<String>> = BTreeMap::new();
    referrers.insert(PathBuf::from("tasks/live.typ"), ids(&["orphan", "gone"]));
    referrers.insert(PathBuf::from("tasks/other.typ"), ids(&["gone"]));

    assert_eq!(
        crate::diag::plain(&reject_diag(&referrers)),
        "error: cannot remove tasks that other tasks link to\n\
         hint: a gitless repository has no tombstone to rewrite onto: set [rm] strategy to \"retire\" or \"unlink\" to remove them anyway\n\
         note: link from: tasks/live.typ to gone, orphan\n\
         note: link from: tasks/other.typ to gone\n"
    );
}

#[test]
fn referrers_maps_files_to_hits_and_skips_eligible() {
    // `live` references `gone`; `unrelated` does not; `a` (also being removed)
    // does but its edge is not waiting for attention.
    let dir = tempdir_with_files(&[
        ("live.typ", &survivor("depends-on(\"gone.typ\")")),
        ("unrelated.typ", "no links\n"),
        ("a.typ", &survivor("related(\"gone.typ\")[why]")),
    ]);
    let root = dir.path();
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
    let dir = tempdir();
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
    let dir = tempdir_with_files(&[
        ("gone.typ", "gone\n"),
        (
            "live.typ",
            &survivor("depends-on(\"gone.typ\") + related(\"gone.typ\")[why]"),
        ),
    ]);
    let root = dir.path();
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
    let dir = tempdir_with_files(&[
        ("gone.typ", "gone\n"),
        ("live.typ", &survivor("depends-on(\"gone.typ\")")),
    ]);
    let root = dir.path();
    let tasks = gone_and_live("done");

    let plan = unlink_plan(&tasks, &ids(&["gone"]), root).unwrap();
    let contents = &plan.rewrites[0].contents;

    assert_eq!(plan.rewrites[0].links, 1);
    assert!(!contents.contains("links:"), "got: {contents}");
    assert!(contents.contains("#show: task.with("));
}

#[test]
fn unlink_plan_sweeps_plus_but_keeps_other_edges() {
    let dir = tempdir_with_files(&[
        ("gone.typ", "gone\n"),
        (
            "live.typ",
            &survivor("depends-on(\"other.typ\") + related(\"gone.typ\")[why]"),
        ),
    ]);
    let root = dir.path();
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
    let dir = tempdir_with_files(&[("gone.typ", "gone\n"), ("live.typ", "no links here\n")]);
    let root = dir.path();
    let tasks = gone_and_live("done");

    let plan = unlink_plan(&tasks, &ids(&["gone"]), root).unwrap();

    assert_eq!(plan.deletions.len(), 1);
    assert!(plan.rewrites.is_empty());
}

#[test]
fn unlink_plan_strips_legacy_link_calls() {
    // The pre-`links:` DSL referenced tasks from body text with `#link("stem.typ")[label]`
    // calls, which `link_edges` never scans — the unlink rewrite carries its own
    // `link(...)` pass. With no `links:` field there is no named argument to
    // drop afterwards, either. A non-literal argument (`#link(gone.typ)`) is
    // not a target: only quoted strings name a task.
    let dir = tempdir_with_files(&[
        ("gone.typ", "gone\n"),
        (
            "live.typ",
            mindtape::typst_task!(
                "(\n  title: \"Live\",\n)\n\n== Summary\n\n\
#link(\"gone.typ\")[Depends on gone]\n\
#link(gone.typ)[Unquoted]\n"
            ),
        ),
    ]);
    let root = dir.path();
    let tasks = gone_and_live("done");

    let plan = unlink_plan(&tasks, &ids(&["gone"]), root).unwrap();
    let contents = &plan.rewrites[0].contents;

    assert_eq!(plan.rewrites.len(), 1);
    assert_eq!(plan.rewrites[0].links, 1);
    assert!(contents.contains("== Summary"), "got: {contents}");
    assert!(!contents.contains("#link(\"gone.typ\")"), "got: {contents}");
    assert!(
        contents.contains("#link(gone.typ)[Unquoted]"),
        "non-literal arg must survive, got: {contents}"
    );
}

#[test]
fn unlink_plan_drops_whole_links_arg_across_whitespace() {
    // The stripped call is followed by whitespace before the separator comma:
    // the whole-value drop must skip the gap, not stop at the call's end and
    // leave a dangling `links: ` key.
    let dir = tempdir_with_files(&[
        ("gone.typ", "gone\n"),
        ("live.typ", &survivor("depends-on(\"gone.typ\") ")),
    ]);
    let root = dir.path();
    let tasks = gone_and_live("done");

    let plan = unlink_plan(&tasks, &ids(&["gone"]), root).unwrap();
    assert_eq!(plan.rewrites[0].links, 1);

    apply(&plan).unwrap();
    let contents = std::fs::read_to_string(root.join("live.typ")).unwrap();

    let expected = mindtape::typst_task!("(\n  title: \"Live\",\n)\n\n== Summary\n\nBody.\n");
    assert_eq!(contents, expected, "the whole links: argument is dropped");
}

// --- batch awareness across strategies ---

#[test]
fn gitless_strategies_share_the_plain_delete_shape() {
    // The batch case: a removed task's only referrer is also being removed, so
    // no survivor references anything — the plain-delete plan applies and no
    // strategy (and no git) is needed.
    let dir = tempdir_with_files(&[
        ("gone.typ", "gone\n"),
        ("a.typ", &survivor("depends-on(\"gone.typ\")")),
    ]);
    let root = dir.path();
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
