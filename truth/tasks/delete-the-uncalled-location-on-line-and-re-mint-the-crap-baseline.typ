#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Delete the uncalled Location::on_line and re-mint the CRAP baseline",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("coverage",),
  estimate: (src-files: 2),
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
      sound-codebase: helps[a dead function leaves the tree and the ratchet compares against current scores],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 6),
)

== Problem

+ Run `just cover`, then `just crap`.
+ 🔴 It prints `21 CRAP regression(s) detected`, led by `Location::on_line` at 30.0 with 0% coverage.


== TO-BE

- 🟢 `just crap` passes on a fresh lcov.


== Changes

- `crates/mindtape/src/violation.rs`: delete `Location::on_line`, which nothing calls.
- `crap-baseline.json`: re-mint, recording the complexity the commits since the last mint added.


== Rollback

Revert the commit; the old baseline and `on_line` come back together.

== Out of scope

- Splitting the functions whose scores rose. Each sits at 89% coverage or more, so its score is its complexity.


== Dogfood

Not applicable, this changes the gate's baseline, not `mt`.
