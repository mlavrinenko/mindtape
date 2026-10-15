#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Merge the crap baseline and same-key outdatty conflicts",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("tooling",),
  estimate: (src-files: 4),
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
      sound-codebase: helps[parallel worktrees land without hand-resolving the CRAP baseline or the outdatty lock],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 15),
)

== Problem

+ Land two `just fork` worktrees that both touched `crates/mindtape-cli/src/cli/mod.rs`, ran `just crap-baseline` and recorded its outdatty group.
+ 🔴 `just land` rebases the second onto main and stops on both files:
  ```
  CONFLICT (content): Merge conflict in crap-baseline.json
  outdatty-lock-merge: outdatty.lock: group cli-docs, source crates/mindtape-cli/src/cli/mod.rs: both sides changed it and neither matches the merged tree
  ```
+ Someone takes main's side, then re-runs `just crap-baseline` and `just outdatty-update <group>` by hand. It happened on every land on 2026-10-15.


== TO-BE

- 🟢 `crap-baseline.json` merges entry by entry, keyed like the ratchet pairs them. A line shift merges silently, and a function both sides re-scored keeps the stricter score, so `just crap` fails on the merged tree if the merged code scores worse.
- 🟢 An `outdatty.lock` hash both sides changed, which neither side's hash matches in the merged tree, gets a hash no file has. The rebase finishes, and `just outdatty-check` reports that group stale for review.


== Changes

- `crates/mindtape-cli/tests/it/crap_baseline_merge_e2e.rs`: the red test first, a rebase over two branches that shifted one file's functions.
- `crates/mindtape-cli/tests/it/outdatty_lock_merge_e2e.rs`: a same-key conflict now finishes the rebase and leaves the group stale.
- `.just/scripts/crap-baseline-merge.sh`: the new driver.
- `.just/scripts/outdatty-lock-merge.sh`: write the unmatched hash in place of a conflict.
- `.gitattributes`, `Justfile` (`install-hooks`), `.mmz/config.yaml`, `outdatty.yaml`: register and scope the driver.
- `docs/src/contributing.typ`: name the driver, then `just docs md`.


== Rollback

- Delete the `crap-baseline.json` line in `.gitattributes`, and git falls back to its text merge.
- Revert the `outdatty-lock-merge.sh` change, and a same-key hash conflicts again.


== Out of scope

- `version` and `algorithm` conflicts in the lock, which no branch changes on its own.
- Changing how `just crap` pairs entries. The driver keys entries the way the ratchet already does.


== Dogfood

This repo's `just land` stops conflicting on both files when two worktrees touched one source.
