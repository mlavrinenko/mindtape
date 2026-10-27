//! `just install-hooks` leaves `.git/config` holding the merge drivers that a
//! checked-out `.gitattributes` names and the recipe registers: it adds the
//! missing ones, drops a driver it once registered that no worktree names any
//! more, and keeps one added by hand. Each test runs the real recipe in a
//! throwaway repo.
//!
//! Unix only: the recipe is a bash script, and the Windows job has no dev
//! shell.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use crate::support::{checkout, shell};
use mindtape::test_support::tempdir;
use std::path::Path;
use tempfile::TempDir;

/// Run `script` in `dir/main`, with the checkout in `$ROOT`, and return what
/// it printed.
fn bash(dir: &Path, script: &str) -> String {
    let out = shell("bash", &dir.join("main"))
        .args(["-ec", script])
        .env("ROOT", checkout())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// A repo whose config holds drivers from earlier runs of the recipe and by
/// hand, beside a worktree on an older branch that still names `old-driver`.
///
/// - `gone`: marked as the recipe's, named by no `.gitattributes`.
/// - `outdatty-lock`: unmarked, as before the marker, and running a script.
/// - `old-driver`: marked, named by the older worktree only.
/// - `mine`: added by hand, named by no `.gitattributes`.
fn repo() -> TempDir {
    let dir = tempdir();
    std::fs::create_dir(dir.path().join("main")).unwrap();
    bash(
        dir.path(),
        r#"git init -q -b main
        git config user.email t@example.com; git config user.name t
        git config commit.gpgsign false
        cp "$ROOT/.gitattributes" .
        git add .gitattributes; git commit -qm base
        git worktree add -q ../older -b older
        echo '*.old merge=old-driver' >> ../older/.gitattributes
        git -C ../older commit -qam older
        git config merge.gone.driver 'false'; git config merge.gone.mindtape true
        git config merge.outdatty-lock.driver 'bash .just/scripts/outdatty-lock-merge.sh %O %A %B'
        git config merge.old-driver.driver 'true'; git config merge.old-driver.mindtape true
        git config merge.mine.driver 'cat'"#,
    );
    dir
}

/// The merge drivers in the config after the recipe ran, sorted.
fn install_and_list(dir: &TempDir) -> Vec<String> {
    let out = bash(
        dir.path(),
        r#"just --justfile "$ROOT/Justfile" --working-directory . install-hooks >/dev/null
        git config --name-only --get-regexp '^merge\..*\.driver$' | sort"#,
    );
    out.lines()
        .map(|line| {
            line.strip_prefix("merge.")
                .and_then(|name| name.strip_suffix(".driver"))
                .unwrap()
                .to_owned()
        })
        .collect()
}

#[test]
fn drops_a_registered_driver_no_worktree_names_and_keeps_the_rest() {
    let dir = repo();
    let drivers = install_and_list(&dir);
    assert_eq!(
        drivers,
        [
            "crap-baseline",
            "jscpd-baseline",
            "mine",
            "old-driver",
            "outdatty"
        ]
    );
}

#[test]
fn drops_a_driver_once_its_worktree_is_gone() {
    let dir = repo();
    install_and_list(&dir);
    bash(dir.path(), "git worktree remove ../older");
    let drivers = install_and_list(&dir);
    assert_eq!(
        drivers,
        ["crap-baseline", "jscpd-baseline", "mine", "outdatty"]
    );
}
