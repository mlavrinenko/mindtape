//! The runs the merge-driver tests share: a driver on three files as git
//! calls it, git's own text merge of the same three, and two branches merged
//! in a throwaway repo that registers the drivers with the real
//! `just install-hooks`.

use super::{checkout, shell};
use mindtape::test_support::tempdir;
use std::process::Output;
use tempfile::TempDir;

/// A temp dir holding `base`, `ours` and `theirs` as files of those names.
fn sides([base, ours, theirs]: [&str; 3]) -> TempDir {
    let dir = tempdir();
    for (name, content) in [("base", base), ("ours", ours), ("theirs", theirs)] {
        std::fs::write(dir.path().join(name), content).unwrap();
    }
    dir
}

fn assert_ran(out: &Output) {
    assert!(
        out.status.success(),
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Run `driver`, a path under the checkout, on the three sides as git does
/// when it merges `path`. Returns what git would keep and the driver's run,
/// which must succeed.
pub fn run_driver(driver: &str, path: &str, three: [&str; 3]) -> (String, Output) {
    let dir = sides(three);
    let out = shell("bash", dir.path())
        .arg(checkout().join(driver))
        .args(["base", "ours", "theirs", path])
        .output()
        .unwrap();
    assert_ran(&out);
    (
        std::fs::read_to_string(dir.path().join("ours")).unwrap(),
        out,
    )
}

/// Whether git's text merge of the three sides conflicts, which is what a
/// merge without the driver would do.
pub fn text_merge_conflicts(three: [&str; 3]) -> bool {
    let dir = sides(three);
    let out = shell("git", dir.path())
        .args(["merge-file", "-p", "ours", "base", "theirs"])
        .output()
        .unwrap();
    !out.status.success()
}

/// In a throwaway repo that carries `driver` and the checkout's
/// `.gitattributes`, commit the base side as `file` on `main`, then theirs on
/// a `side` branch and ours on `main`. With `rebase`, `side` is replayed onto
/// `main`, else `main` merges `side`; ours is `main` either way. Returns
/// `file` after a merge that must succeed.
pub fn merge_branches(driver: &str, file: &str, three: [&str; 3], rebase: bool) -> String {
    let dir = sides(three);
    let script = r#"set -e
        git init -q -b main
        git config user.email t@example.com; git config user.name t
        git config commit.gpgsign false
        mkdir -p "$(dirname "$DRIVER")" "$(dirname "$FILE")"
        cp "$ROOT/$DRIVER" "$DRIVER"
        cp "$ROOT/.gitattributes" .
        just --justfile "$ROOT/Justfile" --working-directory . install-hooks
        cp base "$FILE"
        git add .gitattributes "$DRIVER" "$FILE"; git commit -qm base
        git checkout -qb side; cp theirs "$FILE"; git commit -qam side
        git checkout -q main; cp ours "$FILE"; git commit -qam main
        if [ -n "$REBASE" ]; then git checkout -q side; git rebase -q main
        else git merge -q --no-edit side; fi"#;
    let out = shell("bash", dir.path())
        .args(["-c", script])
        .env("ROOT", checkout())
        .env("DRIVER", driver)
        .env("FILE", file)
        .env("REBASE", if rebase { "1" } else { "" })
        .output()
        .unwrap();
    assert_ran(&out);
    std::fs::read_to_string(dir.path().join(file)).unwrap()
}
