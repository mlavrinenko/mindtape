//! Steps of `.github/workflows/release.yml`, lifted out with `yq` and run on
//! fixtures.
//!
//! "Extract the changelog section" writes the tag's `CHANGELOG.md` section as
//! the GitHub release body. GitHub refuses a body over 125,000 characters, and
//! 0.3.0's section is longer, so a section over the step's budget is cut before
//! the entry that crosses it and ends with a link to the whole file.
//!
//! "Require a green ci.yml run" stands in for a second `just check`: it passes
//! only when `ci.yml` succeeded on the commit the tag names. A stub `gh` on
//! `PATH` answers the run lookup and records what it was asked.
//!
//! Unix only: the steps need `bash`, `awk`, `git` and the dev shell's `yq`.
#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::disallowed_methods)]

use mindtape::test_support::tempdir;
use std::path::Path;

const CHANGELOG: &str = "# Changelog

## [Unreleased]

## [1.2.0] - 2026-10-14

### Upgrading from 1.1

Start over.

### Added

- **First entry.** Some words
  that wrap.
- **Second entry.** More words.

## [1.1.0] - 2026-03-16

### Added

- Older entry.

[1.2.0]: https://github.com/o/r/releases/tag/v1.2.0
";

/// The 1.2.0 section up to the end of its first entry.
const FIRST_ENTRY: &str = "\n### Upgrading from 1.1\n\nStart over.\n\n### Added\n\n\
    - **First entry.** Some words\n  that wrap.\n";

/// Runs the release.yml step `name` in `dir` with `env`: what it printed, and
/// whether it passed.
fn run_step(name: &str, dir: &Path, env: &[(&str, &str)]) -> Result<String, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let script = r#"set -e
        step=$(yq -r --arg name "$STEP" '.jobs[].steps[]? | select(.name == $name) | .run' \
            "$ROOT/.github/workflows/release.yml")
        [ -n "$step" ] || { echo "no step named '$STEP'"; exit 1; }
        bash -ec "$step""#;
    let out = crate::support::shell("bash", dir)
        .args(["-c", script])
        .env("ROOT", &root)
        .env("STEP", name)
        .env("RUNNER_TEMP", dir)
        .env("GITHUB_REPOSITORY", "o/r")
        .envs(env.iter().copied())
        .output()
        .unwrap();
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    match out.status.success() {
        true => Ok(printed),
        false => Err(printed),
    }
}

/// The notes the changelog step writes for `version`, with `budget` if given.
fn notes(version: &str, budget: Option<&str>) -> Result<String, String> {
    let dir = tempdir();
    std::fs::write(dir.path().join("CHANGELOG.md"), CHANGELOG).unwrap();
    let mut env = vec![("VERSION", version)];
    env.extend(budget.map(|b| ("NOTES_BUDGET", b)));
    run_step("Extract the changelog section", dir.path(), &env)?;
    Ok(std::fs::read_to_string(dir.path().join("notes.md")).unwrap())
}

#[test]
fn a_section_under_the_budget_is_the_whole_body() {
    let notes = notes("1.2.0", None).unwrap();
    assert_eq!(
        notes,
        format!("{FIRST_ENTRY}- **Second entry.** More words.\n\n")
    );
}

#[test]
fn a_section_over_the_budget_is_cut_before_the_crossing_entry_and_links_the_file() {
    let notes = notes("1.2.0", Some("100")).unwrap();
    let link = "https://github.com/o/r/blob/v1.2.0/CHANGELOG.md#120---2026-10-14";
    assert_eq!(
        notes,
        format!("{FIRST_ENTRY}\nThe list goes on in [CHANGELOG.md]({link}).\n")
    );
}

#[test]
fn a_version_with_no_section_fails_the_step() {
    let printed = notes("9.9.9", None).unwrap_err();
    assert!(printed.contains("::error::CHANGELOG.md has no section for 9.9.9"));
}

/// Runs the CI gate step in a one-commit repo, with a stub `gh` that answers
/// `answer` and logs its arguments: what the step printed, whether it passed,
/// the commit, and the stub's log.
fn ci_gate(answer: &str) -> (Result<String, String>, String, String) {
    let dir = tempdir();
    let bin = dir.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let stub = format!(
        "#!/bin/sh\necho \"$*\" >> \"{}/gh.log\"\necho '{answer}'\n",
        dir.path().display()
    );
    std::fs::write(bin.join("gh"), stub).unwrap();
    let setup = r"set -e
        chmod +x bin/gh
        git init -q -b main repo; cd repo
        git -c user.email=t@example.com -c user.name=t commit -q --allow-empty -m one
        git rev-parse HEAD";
    let out = crate::support::shell("bash", dir.path())
        .args(["-c", setup])
        .output()
        .unwrap();
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    let env = [("PATH", path.as_str()), ("TAG", "v1.2.0")];
    let result = run_step("Require a green ci.yml run", &dir.path().join("repo"), &env);
    let log = std::fs::read_to_string(dir.path().join("gh.log")).unwrap_or_default();
    (result, sha, log)
}

#[test]
fn the_ci_gate_passes_when_ci_succeeded_on_the_tagged_commit() {
    let (result, sha, log) = ci_gate("42");
    assert!(result.unwrap().contains("ci.yml run 42 passed"));
    assert!(log.contains(&format!("--commit {sha}")), "{log}");
    assert!(log.contains("--workflow ci.yml"), "{log}");
    assert!(log.contains("--status success"), "{log}");
}

#[test]
fn the_ci_gate_fails_when_ci_has_no_green_run_on_the_tagged_commit() {
    let (result, sha, _) = ci_gate("");
    let printed = result.unwrap_err();
    assert!(
        printed.contains(&format!("::error::ci.yml has no successful run on {sha}")),
        "{printed}"
    );
}
