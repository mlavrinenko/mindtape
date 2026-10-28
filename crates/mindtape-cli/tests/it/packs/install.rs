//! `mt packs install`: fetch what the lock pins.

use std::path::Path;

use mindtape::test_support::write_file;

use super::{assert_resolves_pack, config, lock, locked_project, mt_in, tree_hash};
use crate::support;

#[test]
fn install_fetches_a_locked_source_the_load_then_reads() {
    let (dir, commit, sha) = locked_project();
    let sub = dir.path().join("sub");
    std::fs::create_dir(&sub).unwrap();

    let (code, stdout, stderr) = mt_in(dir.path(), &sub, &["packs", "install"]);

    assert_eq!(code, Some(0), "{stderr}");
    let tree = dir
        .path()
        .join(support::CACHE_HOME)
        .join("mindtape/packs")
        .join(&sha);
    let short = &commit[..7];
    let fetched = format!(
        "fetched acme {short} into {}\n",
        mindtape::display_path(&tree)
    );
    assert_eq!(stdout, fetched);
    assert!(tree.join("commits.toml").is_file(), "{stdout}");
    assert_resolves_pack(dir.path());
}

#[test]
fn a_second_install_counts_the_sources_already_cached() {
    let (dir, _, _) = locked_project();
    let (code, _, stderr) = mt_in(dir.path(), dir.path(), &["packs", "install"]);
    assert_eq!(code, Some(0), "{stderr}");

    let (code, stdout, stderr) = mt_in(dir.path(), dir.path(), &["packs", "install"]);

    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(stdout, "1 source up to date\n");
}

#[test]
fn install_json_names_each_source_and_whether_it_fetched() {
    let (dir, commit, sha) = locked_project();

    let (code, stdout, stderr) = mt_in(dir.path(), dir.path(), &["packs", "install", "--json"]);

    assert_eq!(code, Some(0), "{stderr}");
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let expected = serde_json::json!([
        {"source": "acme", "commit": commit, "sha256": sha, "fetched": true}
    ]);
    assert_eq!(json, expected, "{stdout}");
}

/// How a refusal case breaks a [`locked_project`], given its root, commit
/// and tree hash.
type Break = fn(&Path, &str, &str);

#[test]
fn install_refuses_a_lock_it_cannot_restore_and_caches_nothing() {
    let cases: [(&str, Break, &[&str]); 3] = [
        (
            "a lock the sources disagree with",
            |root, _, _| write_file(root, ".mindtape/config.toml", config("v2")),
            &["lock is stale: 'acme'", "mt packs update"],
        ),
        (
            "a tree of another hash",
            |root, commit, _| lock(root, "v1", commit, &tree_hash("other = 1\n")),
            &["fetch pack source: 'acme'", "hash mismatch"],
        ),
        (
            "a commit its ref does not reach",
            |root, _, sha| lock(root, "v1", &"0".repeat(40), sha),
            &["is not on ref 'v1'", "hint: run `mt packs update`"],
        ),
    ];
    for (case, break_it, expected) in cases {
        let (dir, commit, sha) = locked_project();
        break_it(dir.path(), &commit, &sha);

        let (code, _, stderr) = mt_in(dir.path(), dir.path(), &["packs", "install"]);

        assert_eq!(code, Some(1), "{case}: {stderr}");
        for text in expected {
            assert!(stderr.contains(text), "{case}: {stderr}");
        }
        let packs = dir.path().join(support::CACHE_HOME).join("mindtape/packs");
        let cached = std::fs::read_dir(&packs).map_or(0, Iterator::count);
        assert_eq!(cached, 0, "{case}: {stderr}");
    }
}
