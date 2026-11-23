#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Render the goal reason in the task card",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 3,
  ),
  difficulty: 4,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "tooling", "stateless"),
  assignee: "agent",
  status: proposed(2026, 11, 22),
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("lib/prelude.typ"): `Helps`/`Hinders` return only metadata, so the reason is invisible in the rendered card and flattened to plain text.

A compiled task with `- Before #Helps(adopter, to: g)[the reason] after` prints `Before after`, and the `== Goals` section is empty bullets. The harvested reason is the plain string `the reason`.

== TO-BE

- 🟢 The project goal helper returns the marker and a visible reason, so the source and the PDF both read the verdict.
- 🟢 The core prelude's render stays goal-blind, and the project helper owns the visible shape.
- 🟢 A test compiles a task and asserts the reason text appears in the output.

```typ
== Goals

+ #Helps(adopter, to: read-tasks-at-a-glance)[the reason is visible here and in the card].
```

== Changes

- `truth/goal-dsl.typ` after the core extraction, `lib/prelude.typ` today: make `Helps` and `Hinders` return the marker plus the reason content, and decide whether `Neutral` shows the goal names.
- `truth/prelude/2/lib.typ` or the project render hook: expose the visible reason without adding goal names to the core.
- `crates/mindtape-cli/tests/it/goal_markers_e2e.rs`: assert the reason reaches the harvested markers and the compiled task.

== Out of scope

- Reworking the whole card layout. The task fixes the goal line, not the page design.

== Dogfood

A reader of the PDF can see why a task helps a goal, not only the source reader.

== Goals

+ #Helps(adopter, to: read-tasks-at-a-glance)[the reason is visible in both the source and the rendered card].
+ #Neutral(guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

