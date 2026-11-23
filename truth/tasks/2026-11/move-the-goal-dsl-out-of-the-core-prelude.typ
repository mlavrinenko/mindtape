#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Cut every goal helper out of the core prelude",
  priority: ice(
    impact: 9,
    confidence: 0.9,
    ease: 3,
  ),
  difficulty: 5,
  kind: "refactor",
  tags: ("mt", "tooling", "gating", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 23),
)

== AS-IS

+ Open `lib/prelude.typ`.
+ 🔴 It defines the new `adopter`, `maintainer`, `goal`, `Helps`, `Hinders`, `Neutral` and `check-goal-over`, and the old `helps`, `hurts`, `neutral` and `verdicts`. The core API carries a project policy.
+ Read the 0.5.0 snapshot.
+ 🔴 It copies both blocks, and `truth/goal-calls.typ` imports the new names from the leak.

== TO-BE

- 🟢 The 0.5.0 prelude has no goal, audience or verdict helper: no `goal`, `adopter`, `maintainer`, `Helps`, `Hinders`, `Neutral`, `check-goal-over`, `helps`, `hurts`, `neutral` or `verdicts`.
- 🟢 `truth/goal-dsl.typ` restores only the new marker DSL and leaves the old `x-goals` helpers out. `0.4.0` stays frozen for the legacy example and history.
- 🟢 A search of `lib/prelude.typ` and the 0.5.0 snapshot for those names returns nothing. An adopter builds the DSL in a project module.

```
# lib/prelude.typ
#let task(..) = ...
#let edge(..) = ...
# truth/goal-dsl.typ
#let Helps(..) = ...
```

== Changes

- `lib/prelude.typ`, `crates/mindtape/src/loader/preludes/0.5.0/prelude.typ`, `lib/typst.toml`: remove both goal blocks and re-cut the unreleased 0.5.0.
- `truth/goal-dsl.typ`: restore the new `adopter`, `maintainer`, `goal`, `Helps`, `Hinders`, `Neutral` and `check-goal-over`, and leave `helps`, `hurts`, `neutral` and `verdicts` out.
- `truth/goals.typ`, `truth/process-goals.typ`, `truth/goal-calls.typ`, `truth/lib.typ`, `truth/prelude/2/lib.typ`: import the project DSL, not the prelude.
- `crates/mindtape-cli/tests/it/goal_markers_e2e.rs`, `crates/mindtape/src/loader/embedded_tests.rs`: point the fixtures at the project module and keep the 0.5.0 snapshot core-only.
- `examples/api-service/goals.typ`: stay on 0.4.0, or define the old helpers itself if it moves to 0.5.0.

== Out of scope

- The core-surface gate and the docs rewrite. Separate tasks.
- The duplicate-name, render and message fixes in the project DSL. Their own tasks.

== Dogfood

This repo's tasks import generation 2. The prelude they load does not carry a project concept.

== Goals

+ #Helps(adopter, to: keep-the-engine-agnostic)[the prelude keeps the mechanism and the project keeps its goal policy].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

