//! `just integrate` refuses a branch that names a task in `Refs:` and leaves
//! it `wip` at its tip (`.just/scripts/integrate.sh`). The script asks before
//! it reads the gate record, so these runs never reach `mmz`.
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use std::path::{Path, PathBuf};
use std::process::Output;

mod support;

use support::{git, git_commit_all, isolate, project_with_tasks};

/// A committed-clean `main` holding the `wip` task `parser`, and a worktree on
/// the new branch `feature` in its own directory, which the second value owns.
fn repo_with_worktree() -> (tempfile::TempDir, tempfile::TempDir, PathBuf) {
    let main = project_with_tasks(&[("parser.typ", "Write the parser")]);
    git_commit_all(main.path());
    let (holder, wt) = add_worktree(main.path());
    (main, holder, wt)
}

/// A worktree on the new branch `feature`, in a directory the first value owns.
fn add_worktree(main: &Path) -> (tempfile::TempDir, PathBuf) {
    let holder = tempfile::tempdir().unwrap();
    let wt = holder.path().join("feature");
    git(
        main,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "feature",
            wt.to_str().unwrap(),
        ],
    );
    (holder, wt)
}

/// Commit in `wt` with `message`, leaving the tree clean.
fn commit(wt: &Path, message: &str) {
    git(
        wt,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-a",
            "--allow-empty",
            "-m",
            message,
        ],
    );
}

/// Run the integrate script for `feature` from `main`, with `mt` on `PATH`.
fn integrate(main: &Path) -> Output {
    integrate_with(main, None)
}

/// [`integrate`] with `first` ahead of everything else on `PATH`.
fn integrate_with(main: &Path, first: Option<&Path>) -> Output {
    integrate_from(main, first, &[])
}

/// [`integrate_with`] from a shell whose environment already carries `host`,
/// set before the harness's own scrub ([`isolate`]) runs, as a developer's
/// session would carry it into `cargo test`.
fn integrate_from(main: &Path, first: Option<&Path>, host: &[(&str, &str)]) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.just/scripts/integrate.sh");
    let mt_dir = Path::new(env!("CARGO_BIN_EXE_mt")).parent().unwrap();
    let path = format!(
        "{}{}:{}",
        first.map_or(String::new(), |dir| format!("{}:", dir.display())),
        mt_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut command = std::process::Command::new("bash");
    command
        .arg(script)
        .arg("feature")
        .current_dir(main)
        .env("PATH", path)
        .envs(host.iter().copied());
    isolate(&mut command, main);
    command.output().unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// The refusal text of a run that got past the refs check: none of the
/// script's earlier refusals, and no word of this one.
fn past_the_refs_check(text: &str) {
    for earlier in [
        "no worktree",
        "uncommitted changes",
        "not a fast-forward",
        "IS the worktree",
    ] {
        assert!(!text.contains(earlier), "refused early: {text}");
    }
    assert!(!text.contains("still wip"), "{text}");
}

#[test]
fn a_branch_leaving_a_named_task_wip_is_refused() {
    let (main, _holder, wt) = repo_with_worktree();
    commit(&wt, "feat: parse\n\nRefs: parser");
    let out = integrate(main.path());
    let text = stderr(&out);
    assert!(!out.status.success(), "{text}");
    assert!(
        text.contains("still wip at its tip") && text.contains("  parser"),
        "the refusal names the task: {text}"
    );
}

#[test]
fn a_branch_that_closes_the_task_it_names_passes_the_refs_check() {
    let (main, _holder, wt) = repo_with_worktree();
    let task = wt.join("tasks/parser.typ");
    let source = std::fs::read_to_string(&task).unwrap();
    std::fs::write(&task, source.replace("wip(", "done(")).unwrap();
    commit(&wt, "feat: parse\n\nRefs: parser");
    let text = stderr(&integrate(main.path()));
    past_the_refs_check(&text);
}

#[test]
fn a_branch_naming_no_task_passes_the_refs_check() {
    let (main, _holder, wt) = repo_with_worktree();
    commit(&wt, "task: note the parser");
    let text = stderr(&integrate(main.path()));
    past_the_refs_check(&text);
}

/// The script rebuilds the worktree's `mt` with `just build-mt` before its
/// first `mt` call, so the branch is judged by a binary built from its own
/// tip. A stand-in `just` on `PATH` records the call, because the test host
/// need not have the real one, and a pass-through `direnv` keeps the host's own
/// from rewriting `PATH` out from under the stand-in.
#[cfg(unix)]
#[test]
fn the_worktrees_mt_is_rebuilt_before_the_branch_is_judged() {
    use std::os::unix::fs::PermissionsExt;
    let (main, holder, wt) = repo_with_worktree();
    commit(&wt, "task: note the parser");
    let bin = holder.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let marker = holder.path().join("built");
    let shim = bin.join("just");
    std::fs::write(
        &shim,
        format!(
            "#!/bin/sh\n[ \"$1\" = --show ] && exit 0\n[ \"$1\" = build-mt ] && touch {}\n",
            marker.display()
        ),
    )
    .unwrap();
    let direnv = bin.join("direnv");
    std::fs::write(&direnv, "#!/bin/sh\nshift 2\nexec \"$@\"\n").unwrap();
    for script in [&shim, &direnv] {
        std::fs::set_permissions(script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let text = stderr(&integrate_with(main.path(), Some(&bin)));
    assert!(marker.exists(), "no rebuild: {text}");
    past_the_refs_check(&text);
}

/// `direnv exec` rewinds `PATH` to the shell's pre-load value when it inherits
/// `DIRENV_DIFF`/`DIRENV_WATCHES`, which swaps the `mt` under test for the
/// host's. A `direnv` stand-in does the same rewind and a sentinel `mt` sits
/// where the host's would: reaching it fails the run, so a pass means the
/// script ran the binary under test whatever direnv state the host carries.
#[cfg(unix)]
#[test]
fn the_script_runs_the_mt_under_test_whatever_direnv_state_the_host_carries() {
    use std::os::unix::fs::PermissionsExt;
    let (main, holder, wt) = repo_with_worktree();
    commit(&wt, "task: note the parser");
    let host_dir = holder.path().join("host-bin");
    let bin = holder.path().join("bin");
    std::fs::create_dir(&host_dir).unwrap();
    std::fs::create_dir(&bin).unwrap();
    let sentinel = host_dir.join("mt");
    std::fs::write(&sentinel, "#!/bin/sh\necho 'host mt reached' >&2\nexit 1\n").unwrap();
    let direnv = bin.join("direnv");
    let restore = format!(
        "#!/bin/sh\nshift 2\n[ -n \"$DIRENV_DIFF$DIRENV_WATCHES\" ] && PATH={}:$PATH\nexec \"$@\"\n",
        host_dir.display()
    );
    std::fs::write(&direnv, restore).unwrap();
    for script in [&sentinel, &direnv] {
        std::fs::set_permissions(script, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let host = [("DIRENV_DIFF", "x"), ("DIRENV_WATCHES", "x")];
    let text = stderr(&integrate_from(main.path(), Some(&bin), &host));
    assert!(!text.contains("host mt reached"), "{text}");
    past_the_refs_check(&text);
}

/// A scratch repo whose `.mmz/config.yaml` declares one gate-tagged rule over
/// `tasks/**`, a worktree on `feature` where that rule has passed, and a `bin`
/// of pass-through `direnv` and no-op `just` stand-ins. `None` when the host
/// has no `mmz`, the one tool this fixture cannot fake.
#[cfg(unix)]
fn gated_repo() -> Option<(tempfile::TempDir, tempfile::TempDir, PathBuf, PathBuf)> {
    use std::os::unix::fs::PermissionsExt;
    let probe = std::process::Command::new("mmz").arg("--version").output();
    if probe.is_err() {
        eprintln!("skipped: no mmz on PATH");
        return None;
    }
    let main = project_with_tasks(&[("parser.typ", "Write the parser")]);
    std::fs::create_dir(main.path().join(".mmz")).unwrap();
    std::fs::write(
        main.path().join(".mmz/config.yaml"),
        "scopes:\n  tasks: [\"tasks/**\"]\ncommands:\n  - name: \"true\"\n    inputs: [tasks]\n    tags: [gate]\n",
    )
    .unwrap();
    std::fs::write(main.path().join(".gitignore"), ".mmz/cache/\ntarget/\n").unwrap();
    git_commit_all(main.path());
    let (holder, wt) = add_worktree(main.path());
    let bin = holder.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    std::fs::write(bin.join("just"), "#!/bin/sh\nexit 0\n").unwrap();
    std::fs::write(bin.join("direnv"), "#!/bin/sh\nshift 2\nexec \"$@\"\n").unwrap();
    for name in ["just", "direnv"] {
        std::fs::set_permissions(bin.join(name), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    commit(&wt, "task: note the parser");
    let ran = std::process::Command::new("mmz")
        .arg("true")
        .current_dir(&wt)
        .status()
        .unwrap();
    assert!(ran.success());
    Some((main, holder, wt, bin))
}

#[cfg(unix)]
fn main_is_fresh(main: &Path) -> bool {
    std::process::Command::new("mmz")
        .args(["--is-fresh", "--tag", "gate"])
        .current_dir(main)
        .output()
        .unwrap()
        .status
        .success()
}

/// The worktree's gate records are carried into main's cache by the
/// fast-forward, so the next commit on main is not refused for a stale gate.
#[cfg(unix)]
#[test]
fn integrate_carries_the_gate_records_into_main() {
    let Some((main, _holder, _wt, bin)) = gated_repo() else {
        return;
    };
    assert!(!main_is_fresh(main.path()), "main starts stale");
    let out = integrate_with(main.path(), Some(&bin));
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(main_is_fresh(main.path()), "main's gate stayed stale");
    assert!(
        out.stdout.iter().filter(|b| **b == b'\n').count() <= 1,
        "integrate stays quiet: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// mmz stays the judge: a carried record whose input digest does not match
/// main's files is still stale.
#[cfg(unix)]
#[test]
fn a_carried_record_that_does_not_match_main_stays_stale() {
    let Some((main, _holder, _wt, bin)) = gated_repo() else {
        return;
    };
    std::fs::write(main.path().join("tasks/stray.typ"), "x").unwrap();
    let out = integrate_with(main.path(), Some(&bin));
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!main_is_fresh(main.path()));
}

const LCOV: &str = "target/coverage/lcov.info";

/// The worktree's lcov: one record whose `SF:` names `src` under `root`, as
/// `cargo llvm-cov` writes absolute paths.
fn lcov_naming(root: &Path, src: &str) -> String {
    format!("SF:{}/{src}\nDA:1,1\nend_of_record\n", root.display())
}

/// [`gated_repo`] whose branch adds `crates/a.rs`, with the worktree's lcov
/// written after it, so the lcov is fresh there.
#[cfg(unix)]
fn covered_repo() -> Option<(tempfile::TempDir, tempfile::TempDir, PathBuf, PathBuf)> {
    let (main, holder, wt, bin) = gated_repo()?;
    std::fs::create_dir(wt.join("crates")).unwrap();
    std::fs::write(wt.join("crates/a.rs"), "fn a() {}\n").unwrap();
    git(&wt, &["add", "crates/a.rs"]);
    commit(&wt, "feat: a");
    std::fs::create_dir_all(wt.join("target/coverage")).unwrap();
    std::fs::write(wt.join(LCOV), lcov_naming(&wt, "crates/a.rs")).unwrap();
    Some((main, holder, wt, bin))
}

/// What `just crap` would say of main's lcov: empty when it is fresh.
fn lcov_stale_reason(main: &Path) -> String {
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.just/scripts/_lcov-stale-reason.sh");
    let out = std::process::Command::new("bash")
        .arg(script)
        .current_dir(main)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The worktree's lcov lands in main fresh against the fast-forwarded sources,
/// its `SF:` paths moved from the removed worktree onto main.
#[cfg(unix)]
#[test]
fn integrate_carries_the_worktrees_lcov_into_main() {
    let Some((main, _holder, _wt, bin)) = covered_repo() else {
        return;
    };
    let out = integrate_with(main.path(), Some(&bin));
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(lcov_stale_reason(main.path()), "");
    let here = main.path().canonicalize().unwrap();
    assert_eq!(
        std::fs::read_to_string(main.path().join(LCOV)).unwrap(),
        lcov_naming(&here, "crates/a.rs")
    );
}

/// An lcov already stale in the worktree is not carried: a fresh copy would
/// hide that it predates its sources.
#[cfg(unix)]
#[test]
fn a_stale_worktree_lcov_is_not_carried() {
    let Some((main, _holder, wt, bin)) = covered_repo() else {
        return;
    };
    std::fs::write(wt.join("crates/b.rs"), "fn b() {}\n").unwrap();
    git(&wt, &["add", "crates/b.rs"]);
    commit(&wt, "feat: b");
    // Backdated: the kernel stamps mtimes from a coarse clock, so a lcov
    // written in the same tick as `b.rs` would tie with it, not predate it.
    let past = std::time::SystemTime::now() - std::time::Duration::from_secs(60);
    std::fs::File::options()
        .write(true)
        .open(wt.join(LCOV))
        .unwrap()
        .set_modified(past)
        .unwrap();
    let out = integrate_with(main.path(), Some(&bin));
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(!main.path().join(LCOV).exists());
}

/// Main's own lcov, written after the worktree's, is kept.
#[cfg(unix)]
#[test]
fn a_newer_lcov_in_main_is_kept() {
    let Some((main, _holder, _wt, bin)) = covered_repo() else {
        return;
    };
    std::fs::create_dir_all(main.path().join("target/coverage")).unwrap();
    std::fs::write(main.path().join(LCOV), "main's own\n").unwrap();
    let out = integrate_with(main.path(), Some(&bin));
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        std::fs::read_to_string(main.path().join(LCOV)).unwrap(),
        "main's own\n"
    );
}
