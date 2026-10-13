#![allow(clippy::unwrap_used, clippy::indexing_slicing, unsafe_code)]

use mindtape::test_support::write_file;
use mindtape::test_support::{tempdir, tempdir_with_files};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use mindtape::{RmStrategy, parse_config};

use super::super::{
    apply, plain_delete_plan, redact_git_error, referrers, reject_diag, resolve_strategy,
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
    mindtape::typst_task!(
        "(\n  title: \"Gone\",\n  status: wip(2026, 6, 1),\n)\n\n== Summary\n\nBody.\n"
    )
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

// --- retire ---

#[test]
fn retire_plan_defaults_to_first_terminal_status() {
    let dir = tempdir_with_files(&[
        ("gone.typ", &artifact()),
        ("live.typ", &survivor("depends-on(\"gone.typ\")")),
    ]);
    let root = dir.path();
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
    let dir = tempdir_with_files(&[("gone.typ", &artifact())]);
    let root = dir.path();
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
    let dir = tempdir_with_files(&[("gone.typ", "gone\n")]);
    let root = dir.path();
    let config = parse_config("[[tasks.status]]\nvalue = \"wip\"\n").unwrap();
    let tasks = gone_and_live("wip");

    let err = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap_err();

    assert_eq!(
        crate::diag::plain(&err),
        "error: the status legend declares no terminal status\n\
         hint: set [retire] status, or mark a status terminal\n"
    );
}

#[test]
fn retire_plan_names_a_task_file_it_cannot_read() {
    let dir = tempdir();
    let config = parse_config(&legend()).unwrap();
    let tasks = gone_and_live("wip");

    let err = retire_plan(&tasks, &ids(&["gone"]), dir.path(), &config).unwrap_err();

    assert_eq!(
        crate::diag::plain(&err),
        "error: can't read: 'gone.typ': no such file or directory\n"
    );
}

#[test]
fn retire_plan_rename_moves_file_into_archive_folder() {
    let dir = tempdir_with_files(&[("gone.typ", &artifact())]);
    let root = dir.path();
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
    let dir = tempdir_with_files(&[("gone.typ", &artifact()), ("live.typ", &depends_survivor())]);
    let root = dir.path();
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
    let dir = tempdir_with_files(&[("gone.typ", "gone\n")]);
    let root = dir.path();
    let config = parse_config(&legend()).unwrap();
    let tasks = gone_and_live("done");

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();

    assert!(plan.rewrites.is_empty());
    assert!(plan.deletions.is_empty());
}

#[test]
fn retire_plan_applies_without_git() {
    let dir = tempdir_with_files(&[("gone.typ", &artifact())]);
    let root = dir.path();
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

#[test]
fn dbg_whole_links_probe() {
    let source =
        mindtape::typst_task!("(\n  title: \"Live\",\n  links: depends-on(\"gone.typ\") ,\n)\n");
    let call = crate::task_edit::task_with(source).unwrap();
    let value = call.named("links").unwrap();
    eprintln!(
        "value_range: {:?} = {:?}",
        value.value_range,
        &source[value.value_range.clone()]
    );
    let bytes = source.as_bytes();
    for i in value.value_range.clone() {
        eprintln!(
            "  {i}: {} (whitespace={})",
            bytes.get(i).copied().unwrap_or(0),
            bytes.get(i).is_some_and(u8::is_ascii_whitespace)
        );
    }
    let mut cursor = value.value_range.end;
    let mut steps = 0;
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) && steps < 10 {
        cursor += 1;
        steps += 1;
        eprintln!("  cursor {cursor}: {:?}", bytes.get(cursor));
    }
    eprintln!("final cursor {cursor}, byte {:?}", bytes.get(cursor));
}

// --- relative_target, via retire's inbound retarget ---
//
// A survivor in one folder pointing at a retired task in another: the common
// path prefix between `live` and `gone/.archive` is empty, so the rewrite must
// climb with `..`. A `==` -> `!=` mutant in the prefix scan counts the first
// mismatching component as common and drops the climb, yielding a path that no
// longer reaches the moved file.

#[test]
fn retire_rename_retargets_across_folders_with_a_climb() {
    let dir = tempdir();
    let root = dir.path();
    std::fs::create_dir_all(root.join("gone")).unwrap();
    std::fs::create_dir_all(root.join("live")).unwrap();
    write_file(root, "gone/gone.typ", artifact());
    write_file(root, "live/live.typ", depends_survivor());
    let config = parse_config(&format!("{}\n[retire]\nrename = \".archive\"\n", legend())).unwrap();
    let tasks = vec![
        task("gone", "gone/gone.typ", Some("wip")),
        task("live", "live/live.typ", None),
    ];

    let plan = retire_plan(&tasks, &ids(&["gone"]), root, &config).unwrap();

    assert_eq!(plan.link_rewrites.len(), 1);
    let rewritten = &plan.link_rewrites[0];
    assert!(
        rewritten
            .contents
            .contains("depends-on(\"../gone/.archive/gone.typ\")"),
        "the climb out of `live/` is required, got: {}",
        rewritten.contents
    );
}
