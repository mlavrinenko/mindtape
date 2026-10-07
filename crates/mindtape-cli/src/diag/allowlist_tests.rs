use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::UNCONVERTED;

/// Where this file sits in the repository, for `git show`.
const LIST_PATH: &str = "crates/mindtape-cli/src/diag/allowlist.rs";

/// What lets a module bypass the renderer, besides an allow on its `mod` line.
const BYPASS_CALLS: &[&str] = &["Diag::legacy(", "io::stderr()"];

fn src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// `rel` without `.rs`, and a `mod.rs` or `main.rs` as its folder.
fn module_of(rel: &str) -> String {
    let stem = rel.strip_suffix(".rs").unwrap_or(rel);
    match stem.rsplit_once('/') {
        Some((dir, "mod")) => dir.to_owned(),
        None if stem == "main" => String::new(),
        _ => stem.to_owned(),
    }
}

/// Test code may print and build errors as it likes; `diag/` is the renderer.
fn exempt(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    rel.starts_with("diag/")
        || name.ends_with("tests.rs")
        || name.starts_with("tests_")
        || name == "test_util.rs"
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().map(Result::unwrap) {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// The modules that bypass the renderer, as `UNCONVERTED` names them.
fn bypasses() -> BTreeSet<String> {
    let src = src_dir();
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut found = BTreeSet::new();
    for file in files {
        let rel = file
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if exempt(&rel) {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        found.extend(bypasses_in(&module_of(&rel), &text));
    }
    found
}

/// The bypasses in one file of module `module`: an allow on a `mod` line
/// names the declared module, anything else names this one.
fn bypasses_in(module: &str, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut lines = text.lines().map(str::trim);
    while let Some(line) = lines.next() {
        if line.starts_with("//") {
            continue;
        }
        if line.contains("clippy::disallowed_macros") && !line.contains("cfg_attr(test") {
            let declared = lines
                .by_ref()
                .find(|next| !next.starts_with("//") && !next.starts_with("#["))
                .and_then(|next| next.rsplit_once("mod ")?.1.strip_suffix(';'));
            found.push(match declared {
                Some(name) if module.is_empty() => name.to_owned(),
                Some(name) => format!("{module}/{name}"),
                None => module.to_owned(),
            });
        } else if BYPASS_CALLS.iter().any(|call| line.contains(call)) {
            found.push(module.to_owned());
        }
    }
    found
}

fn covers(entry: &str, module: &str) -> bool {
    module == entry || module.starts_with(&format!("{entry}/"))
}

#[test]
fn every_bypass_is_on_the_list() {
    let uncovered: Vec<String> = bypasses()
        .into_iter()
        .filter(|module| !UNCONVERTED.iter().any(|entry| covers(entry, module)))
        .collect();
    assert!(
        uncovered.is_empty(),
        "these modules print or build an error outside crate::diag; convert them \
         to a Diag: {uncovered:?}"
    );
}

#[test]
fn every_entry_still_covers_a_bypass() {
    let found = bypasses();
    let stale: Vec<&str> = UNCONVERTED
        .iter()
        .copied()
        .filter(|entry| !found.iter().any(|module| covers(entry, module)))
        .collect();
    assert!(
        stale.is_empty(),
        "these entries are converted; delete them from diag/allowlist.rs: {stale:?}"
    );
}

#[test]
fn the_list_is_sorted_and_unique() {
    let sorted: BTreeSet<&str> = UNCONVERTED.iter().copied().collect();
    assert!(sorted.iter().copied().eq(UNCONVERTED.iter().copied()));
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The entries of the list as `text` writes them.
fn entries(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix('"')?.strip_suffix("\","))
        .map(str::to_owned)
        .collect()
}

/// The list only shrinks: every entry was on it where this branch left
/// `main`. Skipped where there's no `main` to compare with, as in a shallow
/// CI clone.
#[test]
fn the_list_only_shrinks_from_the_branch_point() {
    let Some(base) = ["main", "origin/main"]
        .iter()
        .find_map(|main| git(&["merge-base", "HEAD", main]))
    else {
        return;
    };
    let Some(before) = git(&["show", &format!("{}:{LIST_PATH}", base.trim())]) else {
        return;
    };
    let before = entries(&before);
    let added: Vec<&str> = UNCONVERTED
        .iter()
        .copied()
        .filter(|entry| !before.contains(*entry))
        .collect();
    assert!(
        added.is_empty(),
        "diag/allowlist.rs only shrinks; build these errors as a Diag instead: {added:?}"
    );
}

#[test]
fn an_allow_on_a_mod_line_names_the_declared_module() {
    let text = "#[allow(clippy::disallowed_macros)]\npub(crate) mod add;\nmod find;\n";
    assert_eq!(bypasses_in("cmd", text), ["cmd/add"]);
    assert_eq!(bypasses_in("", text), ["add"]);
}

#[test]
fn a_bypass_call_names_its_own_module() {
    let text = "// io::stderr() in a comment\nlet mut err = std::io::stderr();\n";
    assert_eq!(bypasses_in("cmd/rm/report", text), ["cmd/rm/report"]);
}

#[test]
fn a_test_only_allow_is_no_bypass() {
    let text = "#![cfg_attr(test, allow(clippy::disallowed_macros))]\n";
    assert!(bypasses_in("", text).is_empty());
}

#[test]
fn an_entry_covers_its_folder_and_not_a_sibling_with_its_prefix() {
    assert!(covers("cmd/rm", "cmd/rm"));
    assert!(covers("cmd/rm", "cmd/rm/gitless/report"));
    assert!(!covers("cmd/rm", "cmd/rmx"));
    assert_eq!(module_of("cmd/rm/mod.rs"), "cmd/rm");
    assert_eq!(module_of("main.rs"), "");
    assert_eq!(module_of("cli/check_argv.rs"), "cli/check_argv");
}
