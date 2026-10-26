//! `mt packs update`: move each source along its `ref` and rewrite the lock.

use std::path::Path;

use mindtape::config::pack_sources::{LockedSource, read_raw};
use mindtape::test_support::{git_as_tester, tempdir_with_config, write_file};

use super::{RULE, Upstream, assert_resolves_pack, config, lock, locked_project, mt_in, tree_hash};

/// `mt packs update <args>` in `root`, which must pass: its stdout.
fn update(root: &Path, args: &[&str]) -> String {
    let words: Vec<&str> = ["packs", "update"]
        .into_iter()
        .chain(args.iter().copied())
        .collect();
    let (code, stdout, stderr) = mt_in(root, root, &words);
    assert_eq!(code, Some(0), "{stderr}");
    stdout
}

/// The lock entry of `acme`.
fn locked(root: &Path) -> Option<LockedSource> {
    read_raw(root).unwrap().lock.entries.remove("acme")
}

/// The lock file's bytes.
fn lock_bytes(root: &Path) -> Vec<u8> {
    std::fs::read(root.join(".mindtape/packs.lock")).unwrap()
}

fn short(commit: &str) -> &str {
    commit.get(..7).unwrap()
}

#[test]
fn update_locks_a_source_the_lock_lacks_and_fetches_it() {
    let dir = tempdir_with_config(&config("v1"));
    let commit = Upstream { root: dir.path() }.publish(RULE, "v1");

    let stdout = update(dir.path(), &[]);

    assert_eq!(
        stdout,
        format!(
            "acme v1: locked {}\n  + acme:commits\nwrote .mindtape/packs.lock\n",
            short(&commit)
        )
    );
    let entry = locked(dir.path()).unwrap();
    assert_eq!(
        (entry.git.as_str(), entry.reference.as_str()),
        ("upstream/src.git", "v1")
    );
    assert_eq!(entry.commit, commit);
    assert_eq!(entry.sha256, tree_hash(RULE));
    assert_resolves_pack(dir.path());
}

/// The rule `acme-rule` after a fix.
const FIXED: &str = "[[tasks.rules]]\nname = \"acme-rule\"\nassert = 'kind != \"\"'\n";

/// A [`locked_project`] whose `v1` then moved to [`FIXED`]: the project, the
/// locked commit and the new one.
fn moved_project() -> (tempfile::TempDir, String, String) {
    let (dir, old, _) = locked_project();
    let new = Upstream { root: dir.path() }.publish(FIXED, "v1");
    (dir, old, new)
}

/// What an update from `old` to `new` of [`moved_project`] prints.
fn moved_report(old: &str, new: &str) -> String {
    format!(
        "acme v1: {} → {}\n  ~ acme:commits\n    ~ rule tasks.rules.acme-rule\n\
         wrote .mindtape/packs.lock\n",
        short(old),
        short(new)
    )
}

#[test]
fn update_follows_a_moved_annotated_tag_to_its_commit() {
    let (dir, old, new) = moved_project();

    let stdout = update(dir.path(), &["acme"]);

    assert_eq!(stdout, moved_report(&old, &new));
    let entry = locked(dir.path()).unwrap();
    assert_eq!(entry.commit, new);
    assert_eq!(entry.sha256, tree_hash(FIXED));
}

#[test]
fn update_takes_a_commit_ref_as_the_commit_itself() {
    let (dir, commit, _) = locked_project();
    write_file(dir.path(), ".mindtape/config.toml", config(&commit));

    let stdout = update(dir.path(), &[]);

    assert!(stdout.starts_with(&format!("acme {commit}: ")), "{stdout}");
    let entry = locked(dir.path()).unwrap();
    assert_eq!((entry.reference, entry.commit), (commit.clone(), commit));
}

#[test]
fn update_keeps_an_unmoved_entry_and_drops_one_no_source_declares() {
    let cases: [(Option<&str>, &str, bool); 2] = [
        (None, "acme v1: up to date\n", true),
        (
            Some(""),
            "dropped acme\nwrote .mindtape/packs.lock\n",
            false,
        ),
    ];
    for (declared, expected, kept) in cases {
        let (dir, _, _) = locked_project();
        if let Some(text) = declared {
            write_file(dir.path(), ".mindtape/config.toml", text);
        }
        let before = lock_bytes(dir.path());

        let stdout = update(dir.path(), &[]);

        assert_eq!(stdout, expected);
        assert_eq!(lock_bytes(dir.path()) == before, kept, "{stdout}");
        assert_eq!(locked(dir.path()).is_some(), kept, "{stdout}");
    }
}

#[test]
fn update_refuses_a_source_or_a_ref_it_cannot_find() {
    let (dir, _, _) = locked_project();
    let before = lock_bytes(dir.path());

    let (code, _, stderr) = mt_in(dir.path(), dir.path(), &["packs", "update", "acmee"]);
    assert_eq!(code, Some(2), "{stderr}");
    assert!(stderr.contains("unknown pack source: 'acmee'"), "{stderr}");
    assert!(stderr.contains("did you mean acme?"), "{stderr}");

    write_file(dir.path(), ".mindtape/config.toml", config("v9"));
    let (code, _, stderr) = mt_in(dir.path(), dir.path(), &["packs", "update"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("'acme'"), "{stderr}");
    assert!(stderr.contains("no ref 'v9'"), "{stderr}");
    assert_eq!(lock_bytes(dir.path()), before);
}

#[test]
fn update_prints_what_the_moved_commit_changes_pack_by_pack() {
    let (dir, old, _) = locked_project();
    let upstream = Upstream { root: dir.path() };
    let ops = "[[tasks.waits]]\nname = \"pr\"\nrun = \"gh pr view\"\n";
    write_file(upstream.work(), "ops.toml", ops);
    let fixed = "[[tasks.rules]]\nname = \"acme-rule\"\nassert = 'kind != \"\"'\n\n\
        [[on.flip]]\nname = \"gate\"\nwhen = \"true\"\nactions = [{ run = \"just check\" }]\n";
    let new = upstream.publish(fixed, "v1");

    let stdout = update(dir.path(), &[]);

    let moved = format!("acme v1: {} → {}\n", short(&old), short(&new));
    let report = "  ~ acme:commits\n\
        \x20   + on.flip.gate\n\
        \x20       run \"just check\"\n\
        \x20   ~ rule tasks.rules.acme-rule\n\
        \x20 + acme:ops\n\
        \x20     run \"gh pr view\"\n";
    assert_eq!(
        stdout,
        format!("{moved}{report}wrote .mindtape/packs.lock\n")
    );
}

#[test]
fn update_still_locks_a_source_whose_old_tree_is_gone() {
    let dir = tempdir_with_config(&config("v1"));
    let commit = Upstream { root: dir.path() }.publish(RULE, "v1");
    lock(dir.path(), "v1", &"0".repeat(40), &"1".repeat(64));

    let (code, stdout, stderr) = mt_in(dir.path(), dir.path(), &["packs", "update"]);

    assert_eq!(code, Some(0), "{stderr}");
    let warning = stderr.lines().next().unwrap_or_default();
    assert!(
        warning.starts_with("warning: can't fetch the old tree of pack source: 'acme'"),
        "{stderr}"
    );
    assert!(stdout.contains("\n  + acme:commits\n"), "{stdout}");
    assert_eq!(stdout.lines().last(), Some("wrote .mindtape/packs.lock"));
    assert_eq!(locked(dir.path()).unwrap().commit, commit);
}

#[test]
fn update_fetches_the_old_tree_from_the_new_git_when_the_source_moved_hosts() {
    let (dir, old, new) = moved_project();
    let upstream = dir.path().join("upstream");
    std::fs::rename(upstream.join("src.git"), upstream.join("moved.git")).unwrap();
    let moved = config("v1").replace("upstream/src.git", "upstream/moved.git");
    write_file(dir.path(), ".mindtape/config.toml", moved);

    let (code, stdout, stderr) = mt_in(dir.path(), dir.path(), &["packs", "update"]);

    assert_eq!((code, stderr.as_str()), (Some(0), ""));
    assert_eq!(stdout, moved_report(&old, &new));
}

#[test]
fn a_failed_update_prints_no_progress_and_keeps_the_lock() {
    let (dir, _, _) = locked_project();
    Upstream { root: dir.path() }
        .publish("[[tasks.rules]]\nname = \"x\"\nassert = \"true\"\n", "v1");
    let zed = "\n[pack-sources.zed]\ngit = \"upstream/src.git\"\nref = \"v9\"\n";
    write_file(dir.path(), ".mindtape/config.toml", config("v1") + zed);
    let before = lock_bytes(dir.path());

    let (code, stdout, stderr) = mt_in(dir.path(), dir.path(), &["packs", "update"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("'zed'"), "{stderr}");
    assert_eq!(stdout, "");
    assert_eq!(lock_bytes(dir.path()), before);
}

#[test]
fn update_reads_no_config_of_a_repository_around_the_project() {
    let dir = tempdir_with_config(&config("v1"));
    let commit = Upstream { root: dir.path() }.publish(RULE, "v1");
    let root = dir.path();
    git_as_tester(root, &["init", "--quiet"]);
    let upstream = format!("{}/", root.join("upstream").display());
    let rewrite = "url.file:///nonexistent/.insteadOf";
    git_as_tester(root, &["config", rewrite, &upstream]);

    let (code, _, stderr) = mt_in(root, root, &["packs", "update"]);

    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(locked(root).unwrap().commit, commit);
}
