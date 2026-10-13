#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "File the sweep's tasks into a folder and bar small dry clones",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("tooling",),
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
      sound-codebase: helps[a sweep batch is small enough to clear and easy to remove once cleared],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Run `just ask-refactoring`.
+ 🔴 The dry arm reports 639 clone families, median gain 50 tokens, because it has no bar. `--write` would file them among 856 tasks in `truth/tasks/`, where 180 closed swept tasks already sit.

== TO-BE

- 🟢 `just ask-refactoring --dry-gain N` reports a dry family only when its extraction saves N tokens or more, 300 by default.
- 🟢 `--write` files every finding under `truth/tasks/swept/`, so `mt ls swept` lists the batch.

== Changes

- `contrib/ask-refactoring.sh`: take `--dry-gain`, file with `--to swept`.
- `contrib/ask-refactoring/dry.sh`: drop families below the bar.
- `Justfile`: the `--dry-gain` arg and its default.
- `.just/scripts/arg-pattern-check.sh`: a bad and a good `--dry-gain`.
- `www/utils/sweep-flag-notes.typ`: why the dry arm has its own bar.

== Rollback

- Revert the commit. Filed tasks stay where they are.

== Out of scope

- Moving or removing the 180 closed swept tasks; that is the owner's call.

== Dogfood

This repo files its next sweep batch under `truth/tasks/swept/`.
