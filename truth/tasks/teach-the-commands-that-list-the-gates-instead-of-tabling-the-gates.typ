#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Teach the commands that list the gates instead of tabling the gates",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (src-files: 3),
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
      sound-codebase: neutral,
      docs-match-code: helps[a gate list read from `just` never drifts from the Justfile],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
  links: depends-on(
    "run-just-check-in-fail-fast-stages-and-gate-coverage-and-crap-in-it.typ",
  ),
)

== Problem

+ Open `CONTRIBUTING.md` at "Quality Gates".
+ 🔴 A 27-row table restates every gate, and nothing there says which gates `just check` runs first.


== TO-BE

- 🟢 The contributing docs show three commands: `just --show check` for the stages, `just --list --unsorted` for every recipe, and `mmz --status --tag gate` for what is fresh.


== Changes

- `docs/src/contributing.typ`: replace the gate table with the three commands.
- `docs/src/contributing/gates.typ`: describe the stages and how to read them from `just`.
- `docs/src/contributing/coverage-crap.typ`: `cover` and `crap` run in `just check`.


== Rollback

Revert the commit to restore the table.

== Out of scope

- `www/utils/gate-notes.typ`, which the docs site still renders.


== Dogfood

This repo's contributors read the gates from `just`.
