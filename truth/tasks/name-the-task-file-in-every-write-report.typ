#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the task file in every write report",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: helps[every write report names a task by its stem, so one task reads the same in every line],
      implementable-tasks: neutral,
      enforced-practice: neutral,
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
  status: done(2026, 10, 16),
)

== Problem

+ Run `mt wip wcw`, where `wcw` is the stem hash of `fix-the-login-redirect`.
+ 🔴 It prints `flipped wcw (Fix the login redirect): proposed → wip`.
+ Run `mt set wcw assignee=ada`.
+ 🔴 It prints `set fix-the-login-redirect (Fix the login redirect): ...`, a different name for the same task.


== TO-BE

- 🟢 Every write command names a task by its stem, whatever the user typed: a stem, a path or a hash.


== Changes

- `crates/mindtape-cli/tests/it/write_report_stem_e2e.rs`: the red test first, each write report naming the stem when given a hash.
- `crates/mindtape-cli/src/util.rs`: `report_change` prints `<verb> <stem> (<title>): <detail>` for every in-place write.
- `crates/mindtape-cli/src/cmd/flip/mod.rs`, `cmd/set.rs`, `cmd/review.rs`: report through it.
- `crates/mindtape-cli/src/cmd/recur/mod.rs`, `cmd/recur/advance.rs`: name an existing task by its stem and a spawned file by its path from the project root.
- `crates/mindtape-cli/src/task_edit.rs`, `pre_write/mod.rs`: the read, edit and checked write that set and review now share.


== Alternatives

- Echo what the user typed everywhere: a hash in a report is unreadable a day later, and `mt set` already names the stem.


== Out of scope

- `mt add`, which introduces a new file by its path and hash.
- `mt mv` and `mt rm`, which report the files they move and delete.


== Dogfood

This repo's `mt wip <hash>` prints the stem it flipped.
