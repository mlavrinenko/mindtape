//! `mt check --commits` across a prelude version this binary does not
//! support: the tree is skipped with one warning, its refs never resolve
//! against an empty task list, and a skipped tip state skips the tip rules.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;

use crate::commit_range_e2e::{FINISH_AT_THE_TIP, check_range, commit_parser, manifest};
use crate::support::{assert_unsupported_prelude, check_commit_msg, commit, commit_repo, git};

/// A rule that reads the parent state's task, so a fabricated empty head
/// fails it.
const PARENT_LOADS: &str = "[[commits.rules]]\n\
     name = \"parent-loads\"\n\
     scope = \"task\"\n\
     assert = 'head(exists)'\n\
     severity = \"error\"\n";

/// A repository whose latest commit writes `tasks/parser.typ` at `0.1.0` and
/// commits it with `subject`.
fn repo_with_old_parser(rules: &str, subject: &str, work: &str) -> tempfile::TempDir {
    let dir = commit_repo(rules);
    commit_parser(dir.path(), "wip(2026, 6, 1)", "0.1.0", subject, work);
    dir
}

/// Assert a skipped-prelude report did not run the `head(exists)` rule.
fn assert_parent_rule_skipped(text: &str) {
    assert_unsupported_prelude(text, "0.1.0");
    assert!(
        text.contains("its parent tree imports @local/mindtape:0.1.0"),
        "the warning names the parent tree: {text}"
    );
    assert!(
        !text.contains("its own tree imports @local/mindtape:0.1.0"),
        "the child does not import the old version: {text}"
    );
    assert!(!text.contains("parent-loads"), "{text}");
}

/// A repository whose parent commit imports `0.1.0` and whose child rewrites
/// the import to the current prelude.
fn migration_repo(rules: &str) -> tempfile::TempDir {
    let dir = repo_with_old_parser(
        rules,
        "feat: use an old prelude\n\nRefs: parser",
        "src/a.rs",
    );
    commit_parser(
        dir.path(),
        "wip(2026, 6, 1)",
        mindtape::PRELUDE_VERSION,
        "feat: migrate the prelude\n\nRefs: parser",
        "src/b.rs",
    );
    dir
}

#[test]
fn a_range_skips_a_tree_whose_prelude_is_not_supported() {
    let dir = repo_with_old_parser(
        FINISH_AT_THE_TIP,
        "feat: parse with an old prelude\n\nRefs: parser",
        "src/lib.rs",
    );

    let (ok, text) = check_range(dir.path(), "HEAD~1..HEAD");

    assert!(ok, "an unsupported past prelude is skipped: {text}");
    assert!(text.contains("warning:"), "{text}");
    assert!(
        text.contains("its own tree imports @local/mindtape:0.1.0"),
        "the warning names the commit's own tree: {text}"
    );
    assert_unsupported_prelude(&text, "0.1.0");
    for absent in ["Refs: `parser`", "finish-named-task-at-tip"] {
        assert!(
            !text.contains(absent),
            "unexpected in a skipped range: {absent}\n{text}"
        );
    }
}

#[test]
fn a_range_checks_a_supported_commit_after_an_unsupported_parent() {
    let dir = migration_repo(PARENT_LOADS);
    commit_parser(
        dir.path(),
        "done(2026, 6, 2)",
        mindtape::PRELUDE_VERSION,
        "feat: finish the parser\n\nRefs: parser",
        "src/c.rs",
    );

    let (ok, text) = check_range(dir.path(), "HEAD~3..HEAD");
    assert!(ok, "{text}");
    assert_unsupported_prelude(&text, "0.1.0");
    assert!(
        text.contains("ok: commit rules pass on 1 commit"),
        "the supported commit after the migration still runs: {text}"
    );
    assert!(
        text.contains("2 commits skipped for an unsupported prelude"),
        "the old tree and its migration child are skipped: {text}"
    );
}

/// A migration commit whose parent tree is unsupported is skipped instead of
/// checked against a fabricated empty parent, so a `head(...)` rule does not
/// false-fail.
#[test]
fn a_migration_commit_after_an_unsupported_parent_is_skipped() {
    let dir = migration_repo(PARENT_LOADS);

    let (ok, text) = check_range(dir.path(), "HEAD~1..HEAD");
    assert!(
        ok,
        "a migration commit after an unsupported parent is skipped: {text}"
    );
    assert_parent_rule_skipped(&text);
}

/// The `commit-msg` hook loads the same parent and index pair, so it skips the
/// task rules too when either state's prelude is unsupported.
#[test]
fn a_migration_commit_message_after_an_unsupported_parent_skips_task_rules() {
    let dir = migration_repo(PARENT_LOADS);
    git(dir.path(), &["reset", "--soft", "HEAD^"]);

    let (ok, text) = check_commit_msg(dir.path(), "feat: migrate the prelude\n\nRefs: parser\n");
    assert!(
        ok,
        "the hook skips task rules when a compared state is unsupported: {text}"
    );
    assert_parent_rule_skipped(&text);
}

/// Two unsupported imports in one tree are two diagnostics, each naming its
/// own version and the set.
#[test]
fn a_range_warns_once_per_unsupported_version() {
    let dir = commit_repo("");
    let dir = dir.path();
    write_file(
        dir,
        "tasks/old-a.typ",
        "#import \"@local/mindtape:0.0.9\": *\n#show: task.with(title: \"A\")\n",
    );
    write_file(
        dir,
        "tasks/old-b.typ",
        "#import \"@local/mindtape:0.1.0\": *\n#show: task.with(title: \"B\")\n",
    );
    git(dir, &["add", "tasks/old-a.typ", "tasks/old-b.typ"]);
    commit(dir, "feat: two old imports\n\nRefs: parser");

    let (ok, text) = check_range(dir, "HEAD~1..HEAD");
    assert!(ok, "{text}");
    assert_eq!(text.matches("warning:").count(), 2, "{text}");
    assert_unsupported_prelude(&text, "0.0.9");
    assert_unsupported_prelude(&text, "0.1.0");
}

#[test]
fn a_range_skips_a_state_whose_own_pin_is_unsupported() {
    let dir = commit_repo("");
    write_file(dir.path(), "lib/typst.toml", manifest("0.1.0"));
    write_file(
        dir.path(),
        "lib/prelude.typ",
        mindtape::test_support::PRELUDE,
    );
    commit_parser(
        dir.path(),
        "wip(2026, 6, 1)",
        "0.1.0",
        "feat: pin an old prelude\n\nRefs: parser",
        "src/lib.rs",
    );
    git(dir.path(), &["add", "lib/typst.toml", "lib/prelude.typ"]);
    commit(dir.path(), "chore: carry the old lib\n\nRefs: parser");

    let (ok, text) = check_range(dir.path(), "HEAD~2..HEAD");
    assert!(ok, "the unsupported state is skipped: {text}");
    assert_unsupported_prelude(&text, "0.1.0");
    assert!(
        !text.contains("Refs: `parser`"),
        "the skipped task is not a missing ref: {text}"
    );
}

/// A skipped migration commit still supplies the tip state when its own tree
/// loaded, so a tip rule reads the newest supported tasks, not an empty set.
#[test]
fn a_supported_newest_stage_supplies_the_tip_state_after_an_unsupported_parent() {
    let dir = commit_repo(FINISH_AT_THE_TIP);
    let dir = dir.path();
    commit_parser(
        dir,
        "wip(2026, 6, 1)",
        mindtape::PRELUDE_VERSION,
        "feat: start the parser\n\nRefs: parser",
        "src/a.rs",
    );
    commit_parser(
        dir,
        "wip(2026, 6, 1)",
        "0.1.0",
        "feat: use an old prelude\n\nRefs: parser",
        "src/b.rs",
    );
    commit_parser(
        dir,
        "wip(2026, 6, 2)",
        mindtape::PRELUDE_VERSION,
        "feat: migrate the prelude\n\nRefs: parser",
        "src/c.rs",
    );

    let (ok, text) = check_range(dir, "HEAD~3..HEAD");

    assert!(
        !ok && text.contains("finish-named-task-at-tip") && text.contains("parser"),
        "the tip rule runs against the newest supported stage: {text}"
    );
}
