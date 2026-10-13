//! `.jscpd-baseline.json` merges as a union of fingerprints through the
//! `jscpd-baseline` merge driver (`.just/scripts/jscpd-baseline-merge.sh`),
//! which `just install-hooks` registers and `.gitattributes` assigns. The test
//! builds a throwaway repo that carries the driver and the attributes, registers
//! the driver with the real recipe, and merges two branches that each minted a
//! different clone.
//!
//! Unix only: the driver needs `bash` and `jq`, which the dev shell provides
//! and the Windows job, which has no dev shell, does not.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::tempdir;
use std::path::Path;

/// A baseline in the shape jscpd writes: `version`, then sorted fingerprints.
fn baseline(fingerprints: &[(&str, u32)]) -> String {
    let body: Vec<String> = fingerprints
        .iter()
        .map(|(key, count)| format!("    \"{key}\": {count}"))
        .collect();
    format!(
        "{{\n  \"version\": 1,\n  \"fingerprints\": {{\n{}\n  }}\n}}\n",
        body.join(",\n")
    )
}

#[test]
fn two_branches_that_each_added_a_clone_merge_to_their_union() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = tempdir();
    let files = [
        ("base.json", baseline(&[("0aa", 1), ("9cc", 1)])),
        ("ours.json", baseline(&[("0aa", 1), ("9cc", 1), ("9dd", 1)])),
        (
            "theirs.json",
            baseline(&[("0aa", 1), ("9cc", 1), ("9ee", 2)]),
        ),
    ];
    for (name, content) in &files {
        std::fs::write(dir.path().join(name), content).unwrap();
    }
    // Both branches change the line after `9cc`, so git's text merge conflicts.
    let script = r#"set -e
        git init -q -b main
        git config user.email t@example.com; git config user.name t
        git config commit.gpgsign false
        mkdir -p .just/scripts
        cp "$ROOT/.just/scripts/jscpd-baseline-merge.sh" .just/scripts/
        cp "$ROOT/.gitattributes" .
        just --justfile "$ROOT/Justfile" --working-directory . install-hooks
        cp base.json .jscpd-baseline.json
        git add .gitattributes .just .jscpd-baseline.json; git commit -qm base
        git checkout -qb side; cp theirs.json .jscpd-baseline.json; git commit -qam side
        git checkout -q main; cp ours.json .jscpd-baseline.json; git commit -qam main
        git merge -q --no-edit side"#;
    let out = crate::support::shell("bash", dir.path())
        .args(["-c", script])
        .env("ROOT", &root)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let merged = std::fs::read_to_string(dir.path().join(".jscpd-baseline.json")).unwrap();
    let want = baseline(&[("0aa", 1), ("9cc", 1), ("9dd", 1), ("9ee", 2)]);
    assert_eq!(merged, want);
}
