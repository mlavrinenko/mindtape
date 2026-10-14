#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the tests/it mod list out of the clone gate",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "gating"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a gate fails only on a clone someone can extract],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 14),
)

== Problem

+ Adding `release_notes_e2e` to `crates/mindtape-cli/tests/it/main.rs` for the release notes step
+ 🔴 #raw("Justfile"): Every new `tests/it` module changes the fingerprint of `main.rs`'s `mod` list, which jscpd reads as a renamed clone of itself, so `just check-dry` fails until `just dry-baseline` re-mints the baseline. A `mod` list has no helper to extract

```
crates/mindtape-cli/tests/it/main.rs:5: renamed clone of crates/mindtape-cli/tests/it/main.rs:6
```
The two latest commits that added a module, `d0ed2340` and `61c4aa15`, both re-minted `.jscpd-baseline.json`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 Adding a module to `tests/it/main.rs` leaves `just check-dry` green with no baseline change.

```just
DRY_FLAGS := "--no-tips -f rust -k 50 --ignore-identifiers --ignore-literals --similarity=0.8 --ignore '**/tests/it/main.rs' --baseline .jscpd-baseline.json"
```


== Changes

- `Justfile`: add the `--ignore` to `DRY_FLAGS`, then re-mint the baseline once.


== Rollback

- Revert the `DRY_FLAGS` line and the baseline.


== Out of scope

- Other clones in the baseline. Each is a real duplicate with its own fix.


== Dogfood

The next e2e test file in this repo lands without a baseline re-mint.
