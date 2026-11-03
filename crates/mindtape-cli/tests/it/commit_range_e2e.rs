//! End-to-end tests for `mt check --commits <range>`
//! (`truth/tasks/check-commit-rules-over-a-commit-range.typ`): every
//! `[[commits.rules]]` rule run on each commit of a range, reading the
//! message, the files and the task versions from the commit and its first
//! parent.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;
use std::path::Path;

use crate::support;

use support::{commit, commit_repo, git, stage_file};

/// The flip-with-work rule `commit_msg_task_e2e.rs` pins for the hook.
const FLIP_WITH_WORK: &str = "[[commits.rules]]\n\
     name = \"flip-with-work\"\n\
     scope = \"task\"\n\
     when = 'into(\"wip\") || into(\"done\")'\n\
     assert = 'named && !tasks_only'\n\
     message = \"start or close a task in a commit that carries its work\"\n\
     severity = \"error\"\n";

/// Rewrite `tasks/parser.typ` with `status`, importing the prelude at
/// `version`.
pub(crate) fn write_parser_version(dir: &Path, status: &str, version: &str) {
    write_file(
        dir,
        "tasks/parser.typ",
        format!(
            "#import \"@local/mindtape:{version}\": *\n\n\
             #show: task.with(\n  title: \"Write the parser\",\n  status: {status},\n  \
             priority: 3,\n)\n"
        ),
    );
}

/// Rewrite `tasks/parser.typ` with `status`, importing the current prelude.
pub(crate) fn write_parser(dir: &Path, status: &str) {
    write_parser_version(dir, status, mindtape::PRELUDE_VERSION);
}

/// Rewrite `tasks/parser.typ` at `version`, stage it with `work`, and commit
/// it with `subject`.
pub(crate) fn commit_parser(dir: &Path, status: &str, version: &str, subject: &str, work: &str) {
    write_parser_version(dir, status, version);
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, work);
    commit(dir, subject);
}

/// The `typst.toml` a `mindtape` package at `version` carries.
pub(crate) fn manifest(version: &str) -> String {
    format!(
        "[package]\nname = \"mindtape\"\nversion = \"{version}\"\nentrypoint = \"prelude.typ\"\n"
    )
}

/// The abbreviated id of `rev`.
pub(crate) fn short(dir: &Path, rev: &str) -> String {
    let out = std::process::Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--short", rev])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// The tip-rule breach for `parser`, led by `rev`'s id.
pub(crate) fn tip_breach(dir: &Path, rev: &str) -> String {
    format!(
        "error: {}: commits.rules.named-tasks-finish-at-the-tip: parser:",
        short(dir, rev)
    )
}

/// `mt check --commits <range>`: whether it exited zero, and its output.
pub(crate) fn check_range(dir: &Path, range: &str) -> (bool, String) {
    check_range_with(dir, range, &[])
}

/// [`check_range`] with extra `check` flags, such as `--check-merges`.
pub(crate) fn check_range_with(dir: &Path, range: &str, flags: &[&str]) -> (bool, String) {
    let mut args = vec!["check", "--commits", range];
    args.extend_from_slice(flags);
    support::run_text(dir, &args)
}

#[test]
fn a_range_with_one_offending_commit_fails_naming_it() {
    let dir = commit_repo(FLIP_WITH_WORK);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/lib.rs");
    commit(dir, "feat: start parsing\n\nRefs: parser");
    write_parser(dir, "done(2026, 6, 2)");
    git(dir, &["add", "tasks/parser.typ"]);
    commit(dir, "task: close the parser");

    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(!ok, "the close-only commit must fail the range: {text}");
    let offender = format!(
        "error: {}: commits.rules.flip-with-work: parser: start or close",
        short(dir, "HEAD")
    );
    assert!(text.contains(&offender), "names the commit: {text}");
    assert!(
        text.contains("note: commit: 'task: close the parser'"),
        "the breach names the rule and the task: {text}"
    );
    assert!(
        !text.contains("feat: start parsing"),
        "the commit carrying its work passes: {text}"
    );
    assert!(
        text.contains("1 commit broke commit rules"),
        "counts commits: {text}"
    );

    let (ok, text) = check_range(dir, "HEAD~2..HEAD~1");
    assert!(ok, "the work commit alone passes: {text}");
}

#[test]
fn a_range_across_a_prelude_bump_loads_each_tree_against_its_own_version() {
    let dir = parent_loads_repo();
    let dir = dir.path();
    let range = "HEAD~1..HEAD";

    // The parent commit holds 0.2.0: the package manifest and the task import
    // that names it.
    write_file(dir, "lib/typst.toml", manifest("0.2.0"));
    write_parser_version(dir, "wip(2026, 6, 1)", "0.2.0");
    git(dir, &["add", "lib/typst.toml", "tasks/parser.typ"]);
    commit(dir, "feat: parse with the old prelude\n\nRefs: parser");

    // The tip commit bumps both to 0.3.0, the version the local pin holds.
    write_file(dir, "lib/typst.toml", manifest("0.3.0"));
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "lib/typst.toml", "tasks/parser.typ"]);
    commit(dir, "feat: bump the prelude\n\nRefs: parser");

    // Only the tip's 0.3.0 is installed: the parent state must load its own
    // 0.2.0 `lib/`, which the range reads from the parent commit.
    let (ok, text) = check_range(dir, range);
    assert!(ok, "{text}");
    assert!(
        !text.contains("warning:"),
        "the parent's 0.2.0 import must resolve its own tree: {text}"
    );
}

/// A rule that refuses a commit whose parent state cannot load the task it
/// changes, so a range check proves the parent tree's own package resolved.
const PARENT_LOADS: &str = "[[commits.rules]]\n\
     name = \"parent-loads\"\n\
     scope = \"task\"\n\
     assert = 'head(exists)'\n\
     severity = \"error\"\n";

/// A repository whose only commit rule refuses a commit whose parent state
/// cannot load the task it changes.
fn parent_loads_repo() -> tempfile::TempDir {
    commit_repo(PARENT_LOADS)
}

#[test]
fn a_subproject_range_reads_its_own_package_tree() {
    let repo = mindtape::test_support::tempdir();
    let root = repo.path();
    let sub = root.join("sub");
    std::fs::create_dir_all(sub.join(".mindtape")).unwrap();
    std::fs::create_dir_all(sub.join("tasks")).unwrap();
    mindtape::test_support::write_lib_package(&sub, mindtape::test_support::PRELUDE);
    write_file(
        &sub,
        ".mindtape/config.toml",
        format!(
            "[tasks]\nroots = [\"tasks\"]\n{}{PARENT_LOADS}",
            mindtape::test_support::LIB_ENTRY
        ),
    );
    write_parser(&sub, "wip(2026, 6, 1)");
    support::git_commit_all(root);

    // The parent commit holds 0.2.0; the tip bumps both to 0.3.0.
    write_file(&sub, "lib/typst.toml", manifest("0.2.0"));
    write_parser_version(&sub, "wip(2026, 6, 1)", "0.2.0");
    git(root, &["add", "sub/lib/typst.toml", "sub/tasks/parser.typ"]);
    commit(root, "feat: parse with the old prelude\n\nRefs: parser");
    write_file(&sub, "lib/typst.toml", manifest("0.3.0"));
    write_parser(&sub, "wip(2026, 6, 1)");
    git(root, &["add", "sub/lib/typst.toml", "sub/tasks/parser.typ"]);
    commit(root, "feat: bump the prelude\n\nRefs: parser");

    // No package root: the state overlay and the working tree's pin must
    // resolve the package without the machine's install.
    let mut command = support::mt(root);
    command.env(mindtape::DATA_HOME_ENV, "");
    command.env("TYPST_PACKAGE_PATH", "");
    command.args(["check", "-w", "sub", "--commits", "HEAD~1..HEAD"]);
    let (ok, text) = support::text_of(&command.output().unwrap());
    assert!(ok, "{text}");
    assert!(!text.contains("warning:"), "{text}");
}

#[test]
fn a_rule_reading_the_working_tree_is_skipped_and_reported() {
    let rules = "[[commits.rules]]\n\
         name = \"flip-unstaged\"\n\
         scope = \"task\"\n\
         when = 'worktree(status) != staged(status)'\n\
         assert = 'miss(refs)'\n\
         message = \"the commit reads the working tree\"\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    let dir = dir.path();
    write_parser(dir, "done(2026, 6, 2)");
    git(dir, &["add", "tasks/parser.typ"]);
    commit(dir, "feat: close\n\nRefs: parser");
    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(ok, "a skipped rule cannot fail the range: {text}");
    assert!(
        text.contains("skip: commits.rules.flip-unstaged"),
        "the skip names the rule: {text}"
    );
}

#[test]
fn a_trailer_naming_no_task_fails_the_commit() {
    let dir = commit_repo("");
    let dir = dir.path();
    stage_file(dir, "src/lib.rs");
    commit(dir, "feat: parse\n\nRefs: tasks/parser.typ");
    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(!ok, "a path trailer is refused in a range too: {text}");
    assert!(
        text.contains(": Refs: `tasks/parser.typ` is a path"),
        "{text}"
    );
}

#[test]
fn a_range_git_cannot_read_fails() {
    let dir = commit_repo("");
    let (ok, text) = check_range(dir.path(), "no-such-rev..HEAD");
    assert!(!ok, "{text}");
    assert!(
        text.starts_with("error: can't read commit range: 'no-such-rev..HEAD': "),
        "names the range: {text}"
    );
}

#[test]
fn a_root_commit_is_read_against_the_empty_tree() {
    // The seed commit adds `parser` as wip: a start, from no task at all.
    let dir = commit_repo(FLIP_WITH_WORK);
    let (ok, text) = check_range(dir.path(), "HEAD");
    assert!(!ok, "{text}");
    assert!(
        text.contains(": commits.rules.flip-with-work: parser:")
            && text.contains("note: commit: 'seed'"),
        "{text}"
    );
}

#[test]
fn each_state_holds_a_task_once() {
    // `parser` differs from HEAD in the parent and is changed by the commit:
    // listed twice, it would load twice.
    let rules = "[[commits.rules]]\n\
         name = \"one-parser\"\n\
         scope = \"task\"\n\
         assert = 'staged(count(priority == 3) == 1) && head(count(priority == 3) == 1)'\n\
         severity = \"error\"\n";
    let dir = commit_repo(rules);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 2)");
    git(dir, &["add", "tasks/parser.typ"]);
    commit(dir, "task: edit\n\nRefs: parser");
    write_parser(dir, "done(2026, 6, 3)");
    git(dir, &["add", "tasks/parser.typ"]);
    commit(dir, "task: close\n\nRefs: parser");
    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(ok, "{text}");
}

/// The rule `mindtape:tasks` ships for the tip of a range.
pub(crate) const FINISH_AT_THE_TIP: &str = "[[commits.rules]]\n\
     name = \"named-tasks-finish-at-the-tip\"\n\
     scope = \"tip\"\n\
     assert = '!(status == \"wip\")'\n\
     message = \"run `mt done` and commit it with the task's last work\"\n\
     severity = \"error\"\n";

#[test]
fn a_range_naming_a_task_left_wip_fails_with_its_stem() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/lib.rs");
    commit(dir, "feat: start parsing\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(!ok, "a task still wip at the tip refuses the range: {text}");
    assert!(
        text.contains(&format!(
            "error: {}: commits.rules.named-tasks-finish-at-the-tip: parser: run `mt done`",
            short(dir, "HEAD")
        )),
        "names the tip, the rule, the stem and the advice: {text}"
    );
}

#[test]
fn a_range_naming_a_task_that_is_done_passes() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "done(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/lib.rs");
    commit(dir, "feat: parse\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(ok, "{text}");
}

#[test]
fn a_task_wip_in_the_middle_but_done_at_the_tip_passes() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/a.rs");
    commit(dir, "feat: start\n\nRefs: parser");
    write_parser(dir, "done(2026, 6, 2)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/b.rs");
    commit(dir, "feat: finish\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(ok, "only the tip's state counts: {text}");
}

#[test]
fn a_stem_named_in_two_commits_is_reported_once() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/a.rs");
    commit(dir, "feat: start\n\nRefs: parser");
    stage_file(dir, "src/b.rs");
    commit(dir, "feat: go on\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(!ok, "{text}");
    assert_eq!(
        text.matches("named-tasks-finish-at-the-tip: parser:")
            .count(),
        1,
        "{text}"
    );
}

#[test]
fn a_task_the_tip_deleted_is_not_left_wip() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    write_parser(dir, "wip(2026, 6, 1)");
    git(dir, &["add", "tasks/parser.typ"]);
    stage_file(dir, "src/a.rs");
    commit(dir, "feat: start\n\nRefs: parser");
    git(dir, &["rm", "-q", "tasks/parser.typ"]);
    commit(dir, "task: drop the parser");

    let (ok, text) = check_range(dir, "HEAD~2..HEAD");
    assert!(ok, "a task the tip lacks cannot be wip there: {text}");
}

#[test]
fn the_pack_lets_a_fixup_through_the_hook_and_refuses_it_in_a_range() {
    let dir = commit_repo("");
    let dir = dir.path();
    let config = dir.join(".mindtape/config.toml");
    let base = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("extends = [\"mindtape:tasks\"]\n{base}")).unwrap();
    git(dir, &["add", "-A"]);
    commit(dir, "chore: extend the pack\n\nChangelog: none");
    stage_file(dir, "src/lib.rs");
    let message = "fixup! feat: list dictionaries\n";

    let (ok, text) = support::check_commit_msg(dir, message);
    assert!(ok, "the hook lets a fixup through: {text}");

    commit(dir, message);
    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(!ok, "the range must refuse the fixup: {text}");
    assert!(text.contains("no-unsquashed-fixup"), "{text}");
    assert!(text.contains("git rebase -i --autosquash"), "{text}");
}

/// A prelude stub whose marker always carries `title`, so a loaded task's
/// title says which package tree served it.
fn titled_prelude(title: &str) -> String {
    format!(
        "#let task(body, ..meta) = [#metadata((marker: \"mindtape.task\", \
         fields: (title: \"{title}\"), extras: (:))) <mindtape>]\n"
    )
}

/// A config pinning `@local/mindtape` to `path`, with a rule that reads the
/// parent state's title and the tip's.
fn pin_move_config(path: &str) -> String {
    format!(
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[typst.local-packages]]\nname = \"mindtape\"\npath = \"{path}\"\n\n\
         [[commits.rules]]\nname = \"parent-pin\"\nscope = \"task\"\nwhen = 'named'\n\
         assert = 'head(title == \"Old pin\") && staged(title == \"New pin\")'\n\
         severity = \"error\"\n"
    )
}

#[test]
fn a_range_across_a_pin_move_loads_an_unchanged_task_from_its_own_tree() {
    let dir = mindtape::test_support::tempdir();
    let root = dir.path();

    // The parent tree pins the package at `old-lib`, and its task file is the
    // one the range leaves untouched.
    write_file(root, "old-lib/typst.toml", manifest("0.3.0"));
    write_file(root, "old-lib/prelude.typ", titled_prelude("Old pin"));
    write_file(root, ".mindtape/config.toml", pin_move_config("old-lib"));
    write_file(
        root,
        "tasks/parser.typ",
        "#import \"@local/mindtape:0.3.0\": *\n\n#show: task.with(title: \"Write the parser\")\n",
    );
    support::git_commit_all(root);

    // The tip moves the pin to `lib` and changes no task file.
    write_file(root, "lib/typst.toml", manifest("0.3.0"));
    write_file(root, "lib/prelude.typ", titled_prelude("New pin"));
    // The old path stays behind at a third title, so the parent must read its
    // own tree's bytes rather than the working tree's copy.
    write_file(root, "old-lib/prelude.typ", titled_prelude("Moved pin"));
    write_file(root, ".mindtape/config.toml", pin_move_config("lib"));
    git(root, &["add", "-A"]);
    commit(root, "feat: move the prelude pin\n\nRefs: parser");

    let (ok, text) = check_range(root, "HEAD~1..HEAD");
    assert!(
        ok,
        "the unchanged task must resolve against the parent's own pin: {text}"
    );
}

#[test]
fn a_range_state_reads_an_ancestor_layer_for_its_roots_and_pin() {
    let dir = mindtape::test_support::tempdir();
    let root = dir.path();

    // The past commit has no project config: its root and pin must come from
    // the ancestor defaults layer, the layer `load_config` already applies.
    write_file(root, "lib/typst.toml", manifest("0.2.0"));
    write_file(root, "lib/prelude.typ", titled_prelude("Ancestor pin"));
    write_file(
        root,
        ".mindtape/defaults.toml",
        "[tasks]\nroots = [\"backlog\"]\n\n\
         [[typst.local-packages]]\nname = \"mindtape\"\npath = \"lib\"\n",
    );
    write_file(
        root,
        "backlog/parser.typ",
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(title: \"Write the parser\")\n",
    );
    support::git_commit_all(root);

    // The tip adds a project config with its own root and a rule that reads
    // the parent state's task, which only the ancestor layer can supply.
    write_file(
        root,
        ".mindtape/config.toml",
        "[tasks]\nroots = [\"tasks\"]\n\n\
         [[commits.rules]]\nname = \"parent-loads\"\nscope = \"task\"\n\
         when = 'named'\nassert = 'head(exists)'\nseverity = \"error\"\n",
    );
    stage_file(root, "src/lib.rs");
    commit(root, "feat: work\n\nRefs: parser");

    let (ok, text) = check_range(root, "HEAD~1..HEAD");
    assert!(ok, "the missing config keeps the ancestor layer: {text}");
}
