//! `.config/jscpd-baseline.json` merges through the `jq` three-way driver
//! `just install-hooks` registers and `.gitattributes` assigns, since jscpd
//! 5.4.3 has no `--merge-baseline`. The test merges two branches that each
//! changed the baseline in a throwaway repo.
//!
//! Unix only: the run needs `bash` and `jscpd`, which the dev shell provides
//! and the Windows job, which has no dev shell, does not.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support::{merge_branches, text_merge_conflicts};

/// A baseline in the shape jscpd writes: `version`, the `jscpd` that minted
/// it, then sorted fingerprints.
fn baseline(jscpd: &str, fingerprints: &[(&str, u32)]) -> String {
    let body: Vec<String> = fingerprints
        .iter()
        .map(|(key, count)| format!("    \"{key}\": {count}"))
        .collect();
    format!(
        "{{\n  \"version\": 1,\n  \"jscpd\": \"{jscpd}\",\n  \"fingerprints\": {{\n{}\n  }}\n}}\n",
        body.join(",\n")
    )
}

#[test]
fn two_branches_merge_to_what_each_added_and_removed() {
    let base = baseline("5.4.3", &[("0aa", 1), ("9cc", 1)]);
    let ours = baseline("5.4.3", &[("0aa", 1), ("9cc", 1), ("9dd", 1)]);
    // Theirs extracted the `0aa` clone and accepted `9ee` twice.
    let theirs = baseline("5.4.3", &[("9cc", 1), ("9ee", 2)]);
    // Both branches change the line after `9cc`, so git's text merge conflicts.
    let three = [base.as_str(), &ours, &theirs];
    assert!(text_merge_conflicts(three));
    let merged = merge_branches(None, ".config/jscpd-baseline.json", three, false);
    let want = baseline("5.4.3", &[("9cc", 1), ("9dd", 1), ("9ee", 2)]);
    assert_eq!(merged, want);
}

#[test]
fn the_minting_version_follows_the_side_that_changed_it() {
    let base = baseline("5.4.3", &[("0aa", 1)]);
    let ours = baseline("5.4.3", &[("0aa", 1), ("9dd", 1)]);
    // Theirs bumped jscpd and re-minted, so the merge takes its version.
    let theirs = baseline("5.4.4", &[("0aa", 1), ("9ee", 1)]);
    let three = [base.as_str(), &ours, &theirs];
    let merged = merge_branches(None, ".config/jscpd-baseline.json", three, false);
    let want = baseline("5.4.4", &[("0aa", 1), ("9dd", 1), ("9ee", 1)]);
    assert_eq!(merged, want);

    // Both bumped it, to different versions: ours stands, and the gate names
    // the mismatch until someone re-mints.
    let ours = baseline("5.4.5", &[("0aa", 1), ("9dd", 1)]);
    let three = [base.as_str(), &ours, &theirs];
    let merged = merge_branches(None, ".config/jscpd-baseline.json", three, false);
    let want = baseline("5.4.5", &[("0aa", 1), ("9dd", 1), ("9ee", 1)]);
    assert_eq!(merged, want);
}
