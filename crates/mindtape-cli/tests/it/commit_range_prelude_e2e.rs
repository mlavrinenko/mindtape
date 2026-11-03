//! `mt check --commits` across a prelude version this binary does not
//! support: the tree is skipped with one warning, its refs never resolve
//! against an empty task list, and a skipped tip state skips the tip rules.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::write_file;

use crate::commit_range_e2e::{FINISH_AT_THE_TIP, check_range, commit_parser, manifest};
use crate::support::{assert_unsupported_prelude, commit, commit_repo, git};

/// A repository whose latest commit writes `tasks/parser.typ` at `0.1.0` and
/// commits it with `subject`.
fn repo_with_old_parser(rules: &str, subject: &str, work: &str) -> tempfile::TempDir {
    let dir = commit_repo(rules);
    commit_parser(dir.path(), "wip(2026, 6, 1)", "0.1.0", subject, work);
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
    assert_unsupported_prelude(&text, "0.1.0");
    for absent in ["Refs: `parser`", "named-tasks-finish-at-the-tip"] {
        assert!(
            !text.contains(absent),
            "unexpected in a skipped range: {absent}\n{text}"
        );
    }
}

#[test]
fn a_range_proves_a_supported_commit_after_an_unsupported_one() {
    let dir = repo_with_old_parser("", "feat: use an old prelude\n\nRefs: parser", "src/a.rs");
    commit_parser(
        dir.path(),
        "done(2026, 6, 2)",
        mindtape::PRELUDE_VERSION,
        "feat: move to the current prelude\n\nRefs: parser",
        "src/b.rs",
    );

    let (ok, text) = check_range(dir.path(), "HEAD~2..HEAD");
    assert!(ok, "{text}");
    assert_unsupported_prelude(&text, "0.1.0");
    assert!(
        text.contains(
            "ok: commit rules pass on 1 commit (1 commit skipped for an unsupported prelude)"
        ),
        "the supported commit still runs: {text}"
    );
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
