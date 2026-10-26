#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::Path;

use super::{VerifiedTrees, verified_tree};
use crate::config::pack_resolve::PackScope;
use crate::config::pack_sources::{LockedSource, PackLock, PackSources, hash};
use crate::test_support::{tempdir, write_file};

/// `acme` locked to the tree `sha` names.
fn locked(sha: &str) -> PackSources {
    let source = LockedSource {
        git: "https://example.com/acme/packs".to_owned(),
        reference: "v1".to_owned(),
        commit: "3f2c1a9e0b7d4c55e1a2b3c4d5e6f708192a3b4c".to_owned(),
        sha256: sha.to_owned(),
    };
    PackSources {
        sources: BTreeMap::new(),
        lock: PackLock {
            entries: BTreeMap::from([("acme".to_owned(), source)]),
        },
    }
}

fn scope<'a>(
    sources: &'a PackSources,
    cache: &'a Path,
    verified: &'a VerifiedTrees,
) -> PackScope<'a> {
    PackScope {
        registry: &[],
        sources,
        project: true,
        cache: Some(cache),
        verified,
    }
}

#[test]
fn a_load_hashes_each_source_once() {
    let tmp = tempdir();
    let staged = tmp.path().join("staged");
    write_file(&staged, "commits.toml", "");
    let sha = hash(&staged).unwrap();
    let tree = tmp.path().join(&sha);
    std::fs::rename(&staged, &tree).unwrap();
    let sources = locked(&sha);
    let verified = VerifiedTrees::default();
    let load = scope(&sources, tmp.path(), &verified);
    assert_eq!(verified_tree("acme", &load).unwrap(), tree);

    std::fs::write(tree.join("commits.toml"), "changed").unwrap();

    assert_eq!(verified_tree("acme", &load).unwrap(), tree);
    let fresh = VerifiedTrees::default();
    assert!(verified_tree("acme", &scope(&sources, tmp.path(), &fresh)).is_err());
}
