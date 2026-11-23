#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Refuse a duplicate goal name when the goal registry is built",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 3,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tooling", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: done(
    2026,
    11,
    23,
  )[forced past failing gate: "mmz --is-fresh --tag gate"],
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("lib/prelude.typ"): two goals with the same `to` collapse to one `known` entry, so a task can pass with one goal unrated.

A product `alpha: goal(to: "shared goal")` and a process `gamma: goal(to: "shared goal")` leave one merged `shared-goal`. A task rating only `gamma` plus another goal prints `ok: 1 task`, exit 0.

== TO-BE

- 🟢 A duplicate name in either registry is a load or check error, not a last-writer-wins merge.
- 🟢 The test covers both duplicate goals and the message says to rename one `to` or split the registries.

== Changes

- `lib/prelude.typ` today, `truth/goal-dsl.typ` after the core extraction: detect a duplicate `name` before building `known`.
- `crates/mindtape-cli/tests/it/goal_markers_e2e.rs` or the project goal tests: the two-registry collision fails.

== Out of scope

- Moving the goal DSL out of the core prelude. A separate task owns that boundary.

== Dogfood

This repo's own registry gets a duplicate-name gate, so a later goal rename cannot silently drop one.

== Goals

+ #Helps(adopter, to: run-checks-at-the-write)[the check refuses an ambiguous goal registry instead of silently under-checking a goal].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

