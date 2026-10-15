//! `crap-baseline.json` merges entry by entry through the `crap-baseline`
//! merge driver (`.just/scripts/crap-baseline-merge.sh`), which
//! `just install-hooks` registers and `.gitattributes` assigns. The rebase test
//! builds a throwaway repo that carries the driver and the attributes and
//! registers the driver with the real recipe. The others run the driver on
//! three files, as git would.
//!
//! Unix only: the driver needs `bash` and `jq`, which the dev shell provides
//! and the Windows job, which has no dev shell, does not.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support::checkout;
use mindtape::test_support::tempdir;
use std::path::Path;
use std::process::Output;

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

/// Run the driver on `base`, `ours` and `theirs` as git would, returning the
/// result git would keep and the driver's output.
fn merge(base: &str, ours: &str, theirs: &str) -> (String, Output) {
    let dir = tempdir();
    for (name, content) in [("O", base), ("A", ours), ("B", theirs)] {
        std::fs::write(dir.path().join(name), content).unwrap();
    }
    let driver = checkout().join(DRIVER);
    let out = crate::support::shell("bash", dir.path())
        .arg(driver)
        .args(["O", "A", "B", "crap-baseline.json"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    (std::fs::read_to_string(dir.path().join("A")).unwrap(), out)
}

fn text_merge_conflicts(base: &str, ours: &str, theirs: &str) -> bool {
    let dir = tempdir();
    for (name, content) in [("O", base), ("A", ours), ("B", theirs)] {
        std::fs::write(dir.path().join(name), content).unwrap();
    }
    let out = crate::support::shell("git", dir.path())
        .args(["merge-file", "-p", "A", "O", "B"])
        .output()
        .unwrap();
    !out.status.success()
}

#[test]
fn a_rebase_over_two_branches_that_shifted_one_files_functions_does_not_conflict() {
    let base = baseline(&[("f", 10, 5.0), ("g", 20, 2.0), ("h", 30, 1.0)]);
    // main adds `e` above the rest, so every line below moves down by five.
    let main = baseline(&[("e", 5, 1.0), ("f", 15, 5.0), ("g", 25, 2.0), ("h", 35, 1.0)]);
    // feat simplifies `f`, which moves the two below it up by two.
    let feat = baseline(&[("f", 10, 4.0), ("g", 18, 2.0), ("h", 28, 1.0)]);
    assert!(text_merge_conflicts(&base, &main, &feat));

    let root = checkout();
    let dir = tempdir();
    for (name, content) in [("base.json", &base), ("main.json", &main), ("feat.json", &feat)] {
        std::fs::write(dir.path().join(name), content).unwrap();
    }
    let script = r#"set -e
        git init -q -b main
        git config user.email t@example.com; git config user.name t
        git config commit.gpgsign false
        mkdir -p .just/scripts
        cp "$ROOT/.just/scripts/crap-baseline-merge.sh" .just/scripts/
        cp "$ROOT/.gitattributes" .
        just --justfile "$ROOT/Justfile" --working-directory . install-hooks
        cp base.json crap-baseline.json
        git add .gitattributes .just crap-baseline.json; git commit -qm base
        git checkout -qb feat; cp feat.json crap-baseline.json; git commit -qam feat
        git checkout -q main; cp main.json crap-baseline.json; git commit -qam main
        git checkout -q feat
        git rebase -q main"#;
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
    let merged = std::fs::read_to_string(dir.path().join("crap-baseline.json")).unwrap();
    // `e` from main, `f`'s new score from feat, and every line both sides
    // moved from main, the side a rebase calls ours.
    let want = baseline(&[("e", 5, 1.0), ("f", 15, 4.0), ("g", 25, 2.0), ("h", 35, 1.0)]);
    assert_eq!(merged, want);
}

#[test]
fn a_function_both_sides_rescored_keeps_the_stricter_score_either_way() {
    let base = baseline(&[("f", 10, 5.0)]);
    let ours = baseline(&[("f", 10, 7.0)]);
    let theirs = baseline(&[("f", 10, 6.0)]);
    let (merged, out) = merge(&base, &ours, &theirs);
    assert_eq!(merged, theirs);
    let note = String::from_utf8_lossy(&out.stderr);
    assert!(
        note.contains("src/a.rs f:") && note.contains("just crap"),
        "the driver names the function and the gate that decides it: {note}"
    );
    let (merged, _) = merge(&base, &theirs, &ours);
    assert_eq!(merged, theirs);
}

#[test]
fn a_function_one_side_removed_stays_only_if_the_other_rescored_it() {
    let base = baseline(&[("f", 10, 5.0), ("g", 20, 2.0)]);
    let ours = baseline(&[]);
    // theirs re-scores `f` and only moves `g`.
    let theirs = baseline(&[("f", 10, 3.0), ("g", 25, 2.0)]);
    let (merged, _) = merge(&base, &ours, &theirs);
    assert_eq!(merged, baseline(&[("f", 10, 3.0)]));
}

#[test]
fn same_named_functions_keep_their_order_when_their_lines_cross() {
    let base = baseline(&[("D::from", 10, 1.0), ("D::from", 20, 3.0)]);
    // ours closes the gap between the two; theirs pushes both down.
    let ours = baseline(&[("D::from", 10, 1.0), ("D::from", 12, 3.0)]);
    let theirs = baseline(&[("D::from", 40, 1.0), ("D::from", 50, 3.0)]);
    let (merged, _) = merge(&base, &ours, &theirs);
    // The first takes theirs' line and the second keeps ours, which would
    // put the second above the first. It goes one line below instead, so
    // the ratchet, which pairs same-named functions in line order, still
    // pairs each with its own score.
    assert_eq!(
        merged,
        baseline(&[("D::from", 40, 1.0), ("D::from", 41, 3.0)])
    );
}

#[test]
fn the_committed_baseline_merged_with_itself_keeps_every_byte() {
    let path = Path::new(&checkout()).join("crap-baseline.json");
    let lock = std::fs::read_to_string(path).unwrap();
    let (merged, _) = merge(&lock, &lock, &lock);
    assert_eq!(merged, lock);
}
