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
  status: done(2026, 10, 6),
  links: depends-on(
    "run-just-check-in-fail-fast-stages-and-gate-coverage-and-crap-in-it.typ",
  ),
)

== Problem

+ Open `CONTRIBUTING.md` at "Quality Gates".
+ 🔴 A 27-row table restates every gate, and nothing there says which gates `just check` runs first.


== TO-BE

- 🟢 The contributing docs show the commands that print the gates: `just --show` for the stages, `just --list --list-submodules` for every gate with what it checks, and `mmz --status --tag gate` for what is fresh.


== Changes

- `docs/src/contributing.typ`: replace the gate table with the commands.
- `docs/src/contributing/gates.typ`: describe the stages and how to read them from `just`.
- `www/utils/gate-notes.typ`: delete; the table was its only reader.
- `.just/scripts/check-doc-facts.sh`: drop the arm that required a gate-notes entry per gate.


== Rollback

Revert the commit to restore the table.

== Out of scope

- `www/gates.jq`, which still checks that every `[gate]` recipe runs in `just check`.


== Dogfood

This repo's contributors read the gates from `just`.
