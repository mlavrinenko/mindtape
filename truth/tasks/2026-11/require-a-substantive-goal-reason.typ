#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Require a substantive goal reason",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  links: (
    (
      task: "rewire-goals-through-typst-markers.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "mt", "gating", "stateless", "no-ui-ux"),
  assignee: "agent",
  status: done(
    2026,
    11,
    23,
  )[forced past failing gate: "mmz --is-fresh --tag gate"],
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("lib/prelude.typ"): `#Helps(adopter, [ ], to: goal)` passes because the harvested reason is a whitespace string, and the `[ ]` comparison is dead.

The check compares the reason with `none`, `""` and content `[ ]`, but the harvest turns content into a plain string. A one-space block arrives as `" "` and passes.

== TO-BE

- 🟢 The check trims a reason before it judges it. An empty or whitespace-only block fails.
- 🟢 A short minimum, such as three words or twenty characters, makes the reason say why, not just that.
- 🟢 The message points at the missing reason instead of a threshold puzzle.

```text
error: goal `read-tasks-at-a-glance` says helps but not why.
Write the reason, not a blank or a single word.
```


== Changes

- `truth/goal-dsl.typ` after the core extraction, `lib/prelude.typ` today: trim and length-check the reason.
- `crates/mindtape-cli/tests/it/goal_markers_e2e.rs`: add `[ ]`, `[   ]` and a one-word reason to the refusal cases.

== Out of scope

- A prose-quality judgment. The check enforces a floor, not a style.

== Dogfood

The next goal verdict says why in a sentence, not one space.

== Goals

+ #Helps(adopter, to: run-checks-at-the-write)[the gate refuses a reason that says nothing].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

