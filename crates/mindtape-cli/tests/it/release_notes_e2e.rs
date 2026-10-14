//! The release workflow's "Extract the changelog section" step writes the
//! tag's `CHANGELOG.md` section as the GitHub release body. GitHub refuses a
//! body over 125,000 characters, and 0.3.0's section is longer, so a section
//! over the step's budget is cut before the entry that crosses it and ends with
//! a link to the whole file. The test lifts the step's `run` out of
//! `.github/workflows/release.yml` with `yq` and runs it on a fixture changelog.
//!
//! Unix only: the step needs `bash`, `awk` and the dev shell's `yq`.
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

/// Runs the step on `CHANGELOG` for `version`, with `budget` if given: the
/// notes it wrote when it passed, else what it printed.
fn run_step(version: &str, budget: Option<&str>) -> Result<String, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let dir = tempdir();
    std::fs::write(dir.path().join("CHANGELOG.md"), CHANGELOG).unwrap();
    let script = r#"set -e
        step=$(yq -r '.jobs.release.steps[] | select(.name == "Extract the changelog section") | .run' \
            "$ROOT/.github/workflows/release.yml")
        bash -ec "$step""#;
    let mut command = crate::support::shell("bash", dir.path());
    command
        .args(["-c", script])
        .env("ROOT", &root)
        .env("VERSION", version)
        .env("RUNNER_TEMP", dir.path())
        .env("GITHUB_REPOSITORY", "o/r");
    if let Some(budget) = budget {
        command.env("NOTES_BUDGET", budget);
    }
    let out = command.output().unwrap();
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    match out.status.success() {
        true => Ok(std::fs::read_to_string(dir.path().join("notes.md")).unwrap()),
        false => Err(printed),
    }
}

#[test]
fn a_section_under_the_budget_is_the_whole_body() {
    let notes = run_step("1.2.0", None).unwrap();
    assert_eq!(
        notes,
        format!("{FIRST_ENTRY}- **Second entry.** More words.\n\n")
    );
}

#[test]
fn a_section_over_the_budget_is_cut_before_the_crossing_entry_and_links_the_file() {
    let notes = run_step("1.2.0", Some("100")).unwrap();
    let link = "https://github.com/o/r/blob/v1.2.0/CHANGELOG.md#120---2026-10-14";
    assert_eq!(
        notes,
        format!("{FIRST_ENTRY}\nThe list goes on in [CHANGELOG.md]({link}).\n")
    );
}

#[test]
fn a_version_with_no_section_fails_the_step() {
    let printed = run_step("9.9.9", None).unwrap_err();
    assert!(printed.contains("::error::CHANGELOG.md has no section for 9.9.9"));
}
