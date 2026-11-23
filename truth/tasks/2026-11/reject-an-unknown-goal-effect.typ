#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Reject an unknown goal effect",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
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
  status: proposed(2026, 11, 22),
)

== AS-IS

+ Reviewing the goal DSL milestone after it landed.
+ 🔴 #raw("lib/prelude.typ"): a hand-written marker with `effect: "sideways"` or no effect passes if the task has another `Helps`.

The check reads `effect` only to test `== "helps"`; it never rejects a value outside `helps`, `hinders` and `neutral`. The helpers cannot emit one, but `markers` is project-owned input.

== TO-BE

- 🟢 A goal marker must carry one of `helps`, `hinders` or `neutral`. Anything else is a breach that quotes the bad effect.
- 🟢 The check's test covers an unknown effect and a missing effect.

```typ
#metadata((marker: "goal", audience: "adopter", goal: "read-tasks-at-a-glance", effect: "sideways", reason: "why"))
```


== Changes

- `truth/goal-dsl.typ` after the core extraction, `lib/prelude.typ` today: validate `effect` before the audience and reason checks.
- `crates/mindtape-cli/tests/it/goal_markers_e2e.rs`: pin both cases.

== Out of scope

- The helper API. The helpers already write a valid effect, and this guards a hand-written marker.

== Dogfood

A project's own marker cannot smuggle an unknown effect past the gate.

== Goals

+ #Helps(adopter, to: run-checks-at-the-write)[the check reads only the three legal effects].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

