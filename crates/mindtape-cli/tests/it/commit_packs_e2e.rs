//! The shipped `mindtape:tweaks` and `mindtape:commits` packs judge a commit
//! as this repository's hand-written rules did: a product change needs a
//! changelog entry, a named task must be started, and a tweak touches one
//! source file.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::Path;

use mindtape::test_support::write_file;

use crate::support::{check_commit_msg, commit_msg_refusal, commit_repo, git};

/// The pack's own product set is every path, so a test narrows it the way a
/// project does.
const COMMITS: &str =
    "extends = [\"mindtape:commits\"]\n\n[dicts.product]\nvalues = [\"src/**\"]\n";

fn stage_work(dir: &Path, files: &[&str]) {
    for file in files {
        write_file(dir, file, "work\n");
        git(dir, &["add", file]);
    }
}

/// A repo whose config holds `extends` at its top, above the fixture's own.
fn repo_extending(extends: &str) -> tempfile::TempDir {
    let dir = commit_repo("");
    let config = dir.path().join(".mindtape/config.toml");
    let base = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("{extends}\n{base}")).unwrap();
    dir
}

#[test]
fn a_product_change_without_a_changelog_entry_is_refused() {
    let dir = repo_extending(COMMITS);
    stage_work(dir.path(), &["src/lib.rs"]);
    let text = commit_msg_refusal(dir.path(), "feat: parse\n");
    assert!(text.contains("changelog-with-product-change"), "{text}");
}

#[test]
fn a_changelog_entry_or_a_none_trailer_or_a_fixup_passes() {
    let dir = repo_extending(COMMITS);
    stage_work(dir.path(), &["src/lib.rs", "CHANGELOG.md"]);
    assert!(check_commit_msg(dir.path(), "feat: parse\n").0);
    git(dir.path(), &["reset", "-q", "CHANGELOG.md"]);
    assert!(check_commit_msg(dir.path(), "feat: parse\n\nChangelog: none\n").0);
    assert!(check_commit_msg(dir.path(), "fixup! feat: parse\n").0);
}

/// Rewrite `tasks/parser.typ` with `edit` and stage it.
fn stage_task(dir: &Path, edit: impl Fn(&str) -> String) {
    let task = dir.join("tasks/parser.typ");
    let text = std::fs::read_to_string(&task).unwrap();
    std::fs::write(&task, edit(&text)).unwrap();
    git(dir, &["add", "tasks/parser.typ"]);
}

#[test]
fn work_naming_a_task_still_proposed_is_refused() {
    let dir = repo_extending("extends = [\"mindtape:commits\"]\n");
    stage_task(dir.path(), |text| text.replace("wip(2026", "proposed(2026"));
    let text = commit_msg_refusal(dir.path(), "task: reword\n\nRefs: parser\n");
    assert!(text.contains("refs-task-is-started"), "{text}");
}

#[test]
fn a_tweak_changing_two_source_files_is_refused() {
    let dir = repo_extending("extends = [\"mindtape:tweaks\"]\n");
    stage_task(dir.path(), |text| {
        text.replace("wip(2026, 6, 1)", "done(2026, 6, 2)")
            .replace("title:", "tags: (\"tweak\",),\n  title:")
    });
    stage_work(dir.path(), &["src/a.rs", "src/b.rs"]);
    let text = commit_msg_refusal(dir.path(), "fix: two files\n\nRefs: parser\n");
    assert!(text.contains("tweak-stays-one-file"), "{text}");
}

#[test]
fn each_pack_alone_passes_mt_check() {
    for pack in ["mindtape:tweaks", "mindtape:commits"] {
        let dir = repo_extending(&format!("extends = [\"{pack}\"]\n"));
        let (ok, text) = crate::support::run_text(dir.path(), &["check"]);
        assert!(ok, "{pack} alone: {text}");
    }
}
