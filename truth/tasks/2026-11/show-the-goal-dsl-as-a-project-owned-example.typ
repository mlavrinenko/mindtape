#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Show the goal DSL as a project-owned example",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("www", "docs", "mt", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 23),
)

== AS-IS

+ Open `www/content/tie-work-to-goals.typ`.
+ 🔴 It says the prelude defines the goal DSL and keeps the examples on `x-goals`.
+ Open `www/content/dsl.typ`.
+ 🔴 The new Goals section shows an import from `@local/mindtape:0.5.0`. The core then appears to carry the policy.

== TO-BE

- 🟢 The core docs say only what the core provides: `field = "markers"` and a project Typst check.
- 🟢 The goals page shows a recipe. A project module declares the goals and the helpers, a `truth/lib.typ`-style wrapper wires the check, and a task rates them. The example project is the copyable source.
- 🟢 The page never says `@local/mindtape` defines goals.

```typ
#import "goals.typ": goals
#let check(markers) = check-goal-over(goals, markers)
```

== Source of truth

- `lib/prelude.typ`: the core's actual surface after the move.
- `truth/goal-dsl.typ`, `truth/goal-calls.typ`, `truth/lib.typ`: a working project-owned implementation.
- `examples/`: a new or updated example project that the page renders.

== Changes

- `www/content/tie-work-to-goals.typ`, `www/content/dsl.typ`, `www/content/self-validating-tasks.typ`: rewrite the goal sections as an extension recipe and link to the example.
- `examples/api-service` or a new `examples/goals`: use the project-owned DSL, or label the old form as a legacy example.
- `README.md`, `skills/mindtape/references/hand-editing.md`: remove the claim that the prelude defines goals.

== Out of scope

- The code move and the gate. Separate tasks.

== Dogfood

This repo's docs point at a project-owned goal DSL, not a core feature.

== Goals

+ #Helps(maintainer, to: match-docs-to-code)[the docs show the same project-owned DSL the code uses].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe).

