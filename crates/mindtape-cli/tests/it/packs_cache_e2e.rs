//! A pack from a declared source resolves from the cache that
//! `$MINDTAPE_CACHE_HOME` moves: `<cache home>/mindtape/packs/<sha256>/`.
#![allow(clippy::unwrap_used)]

use std::path::Path;

use mindtape::config::pack_sources::hash;
use mindtape::test_support::{tempdir_with_config, write_file};

use crate::support;

const CONFIG: &str = "extends = [\"acme:commits\"]\n\n[pack-sources.acme]\n\
    git = \"https://example.com/acme/packs\"\nref = \"v1\"\n";

const RULE: &str = "[[tasks.rules]]\nname = \"acme-rule\"\nassert = 'kind == \"bug\"'\n";

/// Lock `acme` to a tree holding `commits.toml`, and put the tree in the
/// cache when `installed`.
fn project(installed: bool) -> tempfile::TempDir {
    let dir = tempdir_with_config(CONFIG);
    let staged = dir.path().join("staged");
    write_file(&staged, "commits.toml", RULE);
    let sha = hash(&staged).unwrap();
    let lock = format!(
        "[acme]\ngit = \"https://example.com/acme/packs\"\nref = \"v1\"\n\
         commit = \"3f2c1a9e0b7d4c55e1a2b3c4d5e6f708192a3b4c\"\nsha256 = \"{sha}\"\n"
    );
    write_file(dir.path(), ".mindtape/packs.lock", lock);
    if installed {
        let packs = dir.path().join(support::CACHE_HOME).join("mindtape/packs");
        std::fs::create_dir_all(&packs).unwrap();
        std::fs::rename(&staged, packs.join(sha)).unwrap();
    }
    dir
}

fn origin(dir: &Path) -> String {
    let out = support::mt(dir)
        .args(["config", "show", "--origin"])
        .output()
        .unwrap();
    support::text_of(&out).1
}

#[test]
fn config_show_origin_names_a_cached_pack_by_its_source() {
    let dir = project(true);

    let text = origin(dir.path());

    assert!(
        text.contains("rule tasks.rules.acme-rule: pack acme:commits\n"),
        "{text}"
    );
}

#[test]
fn a_source_missing_from_the_cache_says_to_install_it() {
    let dir = project(false);

    let (code, stderr) = support::run_stderr(dir.path(), &["check"]);

    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains("pack source not installed: 'acme'"),
        "{stderr}"
    );
    assert!(stderr.contains("mt packs install"), "{stderr}");
}
