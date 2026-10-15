//! `crap-baseline.json` merges entry by entry through the `crap-baseline`
//! merge driver (`.just/scripts/crap-baseline-merge.sh`), which
//! `just install-hooks` registers and `.gitattributes` assigns. The rebase test
//! merges two branches in a throwaway repo; the others run the driver on three
//! files, as git would.
//!
//! Unix only: the driver needs `bash` and `jq`, which the dev shell provides
//! and the Windows job, which has no dev shell, does not.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support::{checkout, merge_branches, run_driver, text_merge_conflicts};

const DRIVER: &str = ".just/scripts/crap-baseline-merge.sh";

/// A baseline as `just crap-baseline` writes it, every entry in one file,
/// given as `(function, line, crap)` in file order. The cyclomatic score
/// equals the CRAP score, as it does at full coverage.
fn baseline(entries: &[(&str, u32, f64)]) -> String {
    let body: Vec<String> = entries
        .iter()
        .map(|(function, line, crap)| {
            format!(
                "    {{\n      \"file\": \"src/a.rs\",\n      \"function\": \"{function}\",\n      \"line\": {line},\n      \"cyclomatic\": {crap:?},\n      \"coverage\": 100.0,\n      \"crap\": {crap:?},\n      \"crate\": \"a\"\n    }}"
            )
        })
        .collect();
    format!(
        "{{\n  \"$schema\": \"https://example.com/report-v1.json\",\n  \"version\": \"0.3.1\",\n  \"entries\": [\n{}\n  ]\n}}\n",
        body.join(",\n")
    )
}

/// The driver's result for the three sides, as git would keep it.
fn merge(base: &str, ours: &str, theirs: &str) -> String {
    run_driver(DRIVER, "crap-baseline.json", [base, ours, theirs]).0
}

#[test]
fn a_rebase_over_two_branches_that_shifted_one_files_functions_does_not_conflict() {
    let base = baseline(&[("f", 10, 5.0), ("g", 20, 2.0), ("h", 30, 1.0)]);
    // main adds `e` above the rest, so every line below moves down by five.
    let main = baseline(&[
        ("e", 5, 1.0),
        ("f", 15, 5.0),
        ("g", 25, 2.0),
        ("h", 35, 1.0),
    ]);
    // feat simplifies `f`, which moves the two below it up by two.
    let feat = baseline(&[("f", 10, 4.0), ("g", 18, 2.0), ("h", 28, 1.0)]);
    let three = [base.as_str(), &main, &feat];
    assert!(text_merge_conflicts(three));

    let merged = merge_branches(DRIVER, "crap-baseline.json", three, true);
    // `e` from main, `f`'s new score from feat, and every line both sides
    // moved from main, the side a rebase calls ours.
    let want = baseline(&[
        ("e", 5, 1.0),
        ("f", 15, 4.0),
        ("g", 25, 2.0),
        ("h", 35, 1.0),
    ]);
    assert_eq!(merged, want);
}

#[test]
fn a_function_both_sides_rescored_keeps_the_stricter_score_either_way() {
    let base = baseline(&[("f", 10, 5.0)]);
    let ours = baseline(&[("f", 10, 7.0)]);
    let theirs = baseline(&[("f", 10, 6.0)]);
    let (merged, out) = run_driver(DRIVER, "crap-baseline.json", [&base, &ours, &theirs]);
    assert_eq!(merged, theirs);
    let note = String::from_utf8_lossy(&out.stderr);
    assert!(
        note.contains("src/a.rs f:") && note.contains("just crap"),
        "the driver names the function and the gate that decides it: {note}"
    );
    assert_eq!(merge(&base, &theirs, &ours), theirs);
}

#[test]
fn a_function_one_side_removed_stays_only_if_the_other_rescored_it() {
    let base = baseline(&[("f", 10, 5.0), ("g", 20, 2.0)]);
    let ours = baseline(&[]);
    // theirs re-scores `f` and only moves `g`.
    let theirs = baseline(&[("f", 10, 3.0), ("g", 25, 2.0)]);
    assert_eq!(merge(&base, &ours, &theirs), baseline(&[("f", 10, 3.0)]));
}

#[test]
fn same_named_functions_keep_their_order_when_their_lines_cross() {
    let base = baseline(&[("D::from", 10, 1.0), ("D::from", 20, 3.0)]);
    // ours closes the gap between the two; theirs pushes both down.
    let ours = baseline(&[("D::from", 10, 1.0), ("D::from", 12, 3.0)]);
    let theirs = baseline(&[("D::from", 40, 1.0), ("D::from", 50, 3.0)]);
    // The first takes theirs' line and the second keeps ours, which would
    // put the second above the first. It goes one line below instead, so
    // the ratchet, which pairs same-named functions in line order, still
    // pairs each with its own score.
    assert_eq!(
        merge(&base, &ours, &theirs),
        baseline(&[("D::from", 40, 1.0), ("D::from", 41, 3.0)])
    );
}

#[test]
fn the_committed_baseline_merged_with_itself_keeps_every_byte() {
    let lock = std::fs::read_to_string(checkout().join("crap-baseline.json")).unwrap();
    assert_eq!(merge(&lock, &lock, &lock), lock);
}
