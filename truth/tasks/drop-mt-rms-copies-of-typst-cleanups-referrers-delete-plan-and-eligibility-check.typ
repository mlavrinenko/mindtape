#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop mt rm's copies of typst-cleanup's referrers, delete plan and eligibility check",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli", "substrate"),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[mt rm stops keeping three copies of logic the retirement crate owns],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ Read `crates/mindtape-cli/src/cmd/rm/mod.rs`, `gitless/mod.rs` and `eligible.rs`.
+ 🔴 `referrers` repeats typst-cleanup's `Policy::scan` and `Policy::identity` loop, `plain_delete_plan` builds a `Plan` by hand and copies the crate's sort order, and `check_eligible_files` mints every eligible file a second time because `typst_cleanup::plan` stopped at the first file it couldn't mint.
+ 🔴 `mt rm --dry-run` on a dirty referenced task names only that one file, as `task file has uncommitted changes`, while the real run lists every offender as `cannot remove`.


== TO-BE

- 🟢 `mt rm` asks typst-cleanup 0.3 who references what (`referrers`), deletes unreferenced tasks through `delete_plan`, and renders the crate's `Error::Ineligible`, which names every file that can't carry a tombstone.
- 🟢 A dry run refuses a dirty or untracked task with the same list the real run prints.


== Changes

- `crates/mindtape-cli/src/cmd/rm/tests.rs`, `cli_tests.rs`: the red test first, a dry run listing every offender as `cannot remove`.
- `crates/mindtape-cli/Cargo.toml`: typst-cleanup 0.3.0.
- `crates/mindtape-cli/src/cmd/rm/mod.rs`: call `typst_cleanup::referrers` and `delete_plan`; drop the local `referrers` and the pre-check.
- `crates/mindtape-cli/src/cmd/rm/gitless/mod.rs`, `gitless/unlink.rs`: drop `plain_delete_plan`; `reject_diag` takes the crate's sorted sets.
- `crates/mindtape-cli/src/cmd/rm/eligible.rs`: render `Error::Ineligible` instead of minting.
- `crates/mindtape-cli/src/diag/from_lib.rs`: match the new error variants.


== Out of scope

- `redact_git_error`: hiding the tasks root from git's message is mt's own rule about what a CLI shows, not retirement logic.
- The Windows tombstone path workaround from 178765a5: another branch removes it.


== Dogfood

This repo's own `mt rm` runs through the new calls whenever it retires a task.

