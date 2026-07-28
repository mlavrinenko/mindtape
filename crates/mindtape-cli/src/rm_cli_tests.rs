#![allow(clippy::unwrap_used, clippy::indexing_slicing, unsafe_code)]

// ---------------------------------------------------------------------------
// Mutation-regression tests: each targets a specific cargo-mutants survivor in
// rm.rs.  A test passes on the real code and FAILS under the listed mutation.
// All tests use assert_cmd (subprocess) so they are isolated from the
// OnceLock-based worktree override and avoid unsafe code.
// ---------------------------------------------------------------------------

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use predicates::prelude::*;
use swhid_mint::test_support::repo_with;

/// Minimal task Typst source that `load_project` can evaluate without the prelude.
const METADATA_GONE: &str = "\
#metadata((\n\
  marker: \"mindtape.task\",\n\
  fields: (\n\
    title: \"Gone\",\n\
    status: (name: \"done\", date: datetime(year: 2026, month: 6, day: 15), note: none),\n\
    priority: 5,\n\
  ),\n\
  extras: (:),\n\
))\n";

/// Scaffold a project with committed task files and an uncommitted
/// `.mindtape/config.toml`.  Returns the path to the temp-dir root.
fn scaffold(files: &[(&str, &str)]) -> (tempfile::TempDir, PathBuf) {
    let (tmpdir, root) = repo_with(files);
    std::fs::create_dir_all(root.join(".mindtape")).unwrap();
    std::fs::write(
        root.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    (tmpdir, root)
}

/// Scaffold a project with the in-tree lib package installed (needed for
/// survivor files that `#import "@local/mindtape"`).
fn scaffold_with_lib(gone_src: &str, live_src: Option<&str>) -> (tempfile::TempDir, PathBuf) {
    let (tmpdir, root) = repo_with(&[("seed.txt", "seed\n")]);
    mindtape::test_support::write_lib_package(&root, include_str!("../../../lib/prelude.typ"));
    std::fs::create_dir_all(root.join("tasks")).unwrap();
    std::fs::write(root.join("tasks/gone.typ"), gone_src).unwrap();
    if let Some(live) = live_src {
        std::fs::write(root.join("tasks/live.typ"), live).unwrap();
    }
    swhid_mint::test_support::git(&root, &["add", "-A"]);
    swhid_mint::test_support::git(&root, &["commit", "-q", "-m", "seedwithlib"]);
    std::fs::create_dir_all(root.join(".mindtape")).unwrap();
    std::fs::write(
        root.join(".mindtape/config.toml"),
        "[tasks]\nroots = [\"tasks\"]\n",
    )
    .unwrap();
    (tmpdir, root)
}

fn mt(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("mt").unwrap();
    // The same isolation `tests/support/mod.rs` gives every e2e spawn, spelled
    // out because a unit test cannot reach that module: `mt` layers
    // `dirs::config_dir()/mindtape/defaults.toml` under the project config, so
    // an un-isolated child reads the config of whoever runs the suite.
    cmd.current_dir(root)
        .env("XDG_CONFIG_HOME", root.join("no-global-config"))
        .env("HOME", root)
        .env_remove("MINDTAPE_NOW")
        .env_remove("MINDTAPE_WORKTREE");
    cmd
}

// --- confirm ---

#[test]
fn confirm_accepts_y() {
    // replace confirm -> Result<bool> with Ok(false)
    // replace || with &&
    // replace == with != (first "y")
    let (_dir, root) = scaffold(&[("tasks/gone.typ", METADATA_GONE)]);
    mt(&root)
        .args(["rm", "gone"])
        .write_stdin("y\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted").from_utf8());
}

#[test]
fn confirm_accepts_yes() {
    // replace confirm -> Result<bool> with Ok(false)
    // replace || with &&
    // replace == with != (on "yes")
    let (_dir, root) = scaffold(&[("tasks/gone.typ", METADATA_GONE)]);
    mt(&root)
        .args(["rm", "gone"])
        .write_stdin("yes\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("deleted").from_utf8());
}

// --- print_deletions ---

#[test]
fn print_deletions_prints_deletions() {
    // replace print_deletions with ()
    let (_dir, root) = scaffold(&[("tasks/gone.typ", METADATA_GONE)]);
    mt(&root)
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .success()
        .stdout(predicate::str::contains("gone.typ").from_utf8());
}

// --- report_plan ---

#[test]
fn report_plan_with_rewrites_includes_rewrite_section() {
    // replace report_plan with ()
    // delete ! in report_plan
    let (_dir, root) = scaffold_with_lib(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Gone\",\n  status: done(2026, 6, 1),\n)\n\nBody.\n",
        Some(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Live\",\n  status: wip(2026, 6, 1),\n  \
             links: depends-on(\"gone.typ\"),\n)\n\nBody.\n",
        ),
    );
    mt(&root)
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("would delete")
                .from_utf8()
                .and(predicate::str::contains("would rewrite").from_utf8()),
        );
}

#[test]
fn report_plan_without_rewrites_omits_rewrite_section() {
    // delete ! in report_plan — mutant prints section when empty
    let (_dir, root) = scaffold(&[("tasks/gone.typ", METADATA_GONE)]);
    mt(&root)
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("would delete")
                .from_utf8()
                .and(predicate::str::contains("would rewrite").from_utf8().not()),
        );
}

// --- report_applied ---

#[test]
fn report_applied_with_rewrites_includes_rewrite_section() {
    // replace report_applied with ()
    // delete ! in report_applied
    let (_dir, root) = scaffold_with_lib(
        "#import \"@local/mindtape:0.2.0\": *\n\n\
         #show: task.with(\n  title: \"Gone\",\n  status: done(2026, 6, 1),\n)\n\nBody.\n",
        Some(
            "#import \"@local/mindtape:0.2.0\": *\n\n\
             #show: task.with(\n  title: \"Live\",\n  status: wip(2026, 6, 1),\n  \
             links: depends-on(\"gone.typ\"),\n)\n\nBody.\n",
        ),
    );
    mt(&root)
        .args(["rm", "-y", "gone"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("deleted")
                .from_utf8()
                .and(predicate::str::contains("rewrote").from_utf8()),
        );
}

#[test]
fn report_applied_without_rewrites_omits_rewrite_section() {
    // delete ! in report_applied — mutant prints section when empty
    let (_dir, root) = scaffold(&[("tasks/gone.typ", METADATA_GONE)]);
    mt(&root)
        .args(["rm", "-y", "gone"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("deleted")
                .from_utf8()
                .and(predicate::str::contains("rewrote").from_utf8().not()),
        );
}

// --- run ---

#[test]
fn run_dry_run_skips_preflight() {
    // Mutant deletes `!` so `if args.dry_run { check... }`.
    // Real code skips the pre-flight on dry-run; mutant runs it.  With a dirty
    // eligible file the real code fails from plan() (mint_rev_path on dirty
    // file) — error does NOT contain "uncommitted".  The mutant fails from
    // check_eligible_files — error DOES contain "uncommitted".
    let (_dir, root) = scaffold(&[("tasks/gone.typ", METADATA_GONE)]);
    // Dirty the eligible file.
    std::fs::write(
        root.join("tasks/gone.typ"),
        format!("{METADATA_GONE}// dirty\n"),
    )
    .unwrap();

    mt(&root)
        .args(["rm", "--dry-run", "-y", "gone"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("uncommitted").from_utf8().not());
}

#[test]
fn run_aborts_on_declined_confirm() {
    // Mutant deletes `!` before `confirm(&plan)?` so
    // `if !args.yes && confirm(&plan)?` — real code bails when confirm
    // returns false, mutant does NOT bail (proceeds to apply).
    let (_dir, root) = scaffold(&[("tasks/gone.typ", METADATA_GONE)]);
    mt(&root)
        .args(["rm", "gone"])
        .write_stdin("n\n")
        .assert()
        .failure()
        .stderr(predicate::str::contains("aborted by user").from_utf8());
}
