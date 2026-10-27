#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the outdatty workarounds that outdatty 0.10.0 makes needless",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 8),
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
      sound-codebase: helps[the lock merges and checks through outdatty itself, with no bash copy of its format to keep in step],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 28),
)

== Problem

+ Read `.just/scripts/outdatty-check.sh` and `.just/scripts/outdatty-lock-merge.sh`.
+ 🔴 The first diffs the lock's groups against the manifest's with `yq`, though `outdatty check` has failed on an orphaned lock group since 0.8.0. The second is a 280-line bash merge driver for `.config/outdatty.lock` that needs `yq`, `jq` and `b3sum`, plus a 360-line e2e test, because outdatty shipped no driver.

== TO-BE

- 🟢 The dev shell carries outdatty 0.10.0, and `.gitattributes` assigns its `outdatty merge-lock` driver to `.config/outdatty.lock`, which `just install-hooks` registers.
- 🟢 `just outdatty check` runs `outdatty check` alone.
- 🟢 No script, test or dev-shell package in this repo exists only to merge or check the lock.

== Changes

- `flake.lock`: bump `qahq/outdatty` to the v0.10.0 release commit.
- `.just/scripts/outdatty-check.sh`, `.just/scripts/outdatty-lock-merge.sh`, `crates/mindtape-cli/tests/it/outdatty_lock_merge_e2e.rs`: delete, and drop the test's `mod` from `tests/it/main.rs`.
- `.just/outdatty.just`: `check` runs `outdatty check`.
- `Justfile` (`install-hooks`), `.gitattributes`: register and assign `outdatty merge-lock %O %A %B`.
- `flake.nix`: drop `b3sum`, which only the bash driver used.
- `.mmz/config.yaml`, `.config/outdatty.yaml`: drop the deleted files' scopes and sources.
- `docs/src/contributing.typ`, `docs/src/contributing/windows.typ`, `.github/workflows/ci.yml`, the trailer scripts' comments: name outdatty's driver, then `just docs md`.

== Rollback

- Revert the commit. The lock returns to outdatty 0.9.0 and the bash driver and its test come back.

== Out of scope

- `.just/scripts/outdatty-review-trailer.sh`. The `Docs-Unaffected:` trailer is this repo's commit policy, read from commit messages and amends, and outdatty knows nothing of git.
- The merged-tree match the bash driver tried for a hash both sides changed. outdatty leaves every such hash for review. On a main that passes `outdatty check`, a source both sides changed was edited by both, so only a directed group's dependent could have matched, and `check` reads no such hash.

== Dogfood

The next two worktrees that re-record different outdatty groups land through `just land` with the lock merged by `outdatty merge-lock`.
