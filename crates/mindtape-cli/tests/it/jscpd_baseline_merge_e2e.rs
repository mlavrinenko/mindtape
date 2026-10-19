//! `.config/jscpd-baseline.json` merges as a union of fingerprints through the
//! `jscpd-baseline` merge driver (`.just/scripts/jscpd-baseline-merge.sh`),
//! which `just install-hooks` registers and `.gitattributes` assigns. The test
//! merges two branches that each minted a different clone in a throwaway repo.
//!
//! Unix only: the driver needs `bash` and `jq`, which the dev shell provides
//! and the Windows job, which has no dev shell, does not.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support::{merge_branches, text_merge_conflicts};

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
    let base = baseline(&[("0aa", 1), ("9cc", 1)]);
    let ours = baseline(&[("0aa", 1), ("9cc", 1), ("9dd", 1)]);
    let theirs = baseline(&[("0aa", 1), ("9cc", 1), ("9ee", 2)]);
    // Both branches change the line after `9cc`, so git's text merge conflicts.
    let three = [base.as_str(), &ours, &theirs];
    assert!(text_merge_conflicts(three));
    let merged = merge_branches(
        ".just/scripts/jscpd-baseline-merge.sh",
        ".config/jscpd-baseline.json",
        three,
        false,
    );
    let want = baseline(&[("0aa", 1), ("9cc", 1), ("9dd", 1), ("9ee", 2)]);
    assert_eq!(merged, want);
}
