#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Define the goal DSL over metadata markers",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("mt", "tests", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: helps[a wrong audience is a check breach and a typo is a Typst eval error.],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 22),
  links: parent("rewire-goals-through-typst-markers.typ")
    + depends-on(
      "expose-task-body-markers-to-check-functions.typ",
      "decide-the-goal-set-for-an-agnostic-engine.typ",
    ),
)

== AS-IS

+ 🔴 `truth/lib.typ` `check-goals` reads an `x-goals` dict of strings. A goal is a key, not a Typst reference.
+ Write `#Helps(adopter, to: read-tasks-at-a-glance)[...]` in a task body. 🔴 The prelude doesn't define it. The only way to rate a goal is the header dict.
+ Rate a maintainer goal for the adopter. 🔴 The old check compares the goal's key with the registry, never the audience the author wrote.

== TO-BE

- 🟢 `truth/goal-dsl.typ` defines `adopter`, `maintainer`, `goal`, `Helps`, `Hinders` and `Neutral`. `Helps` takes `to:`, `Hinders` takes `from:`, the reason is the trailing block, and `Neutral` takes the goals it rates neutral.
- 🟢 A goal value carries both spellings: `to:` is the infinitive and `from:` the gerund. The name is the kebab of the infinitive.
- 🟢 Both registries add one goal value per key. `goal-calls.typ` re-exports the DSL, both sets and every goal value, and a task imports it with `*`, so a goal name is in scope as written.
- 🟢 A task writes `== Goals` and one line per verdict: `#Helps(adopter, to: <goal>)[reason]`, `#Hinders(maintainer, from: <goal>)[reason]`, or `#Neutral(<goal>, ...)`.
- 🟢 The check refuses a missing, unknown or duplicate goal, a wrong audience, an empty reason and no helps.

```typ
== Goals

+ #Helps(adopter, to: read-tasks-at-a-glance)[because a newcomer reads the block in 30 seconds].
+ #Hinders(maintainer, from: keep-changes-safe)[because a hand-merged marker slips past the gate].
+ #Neutral(guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
```

== Changes

- `truth/goal-dsl.typ`: `adopter`, `maintainer`, `marker(d)`, `goal(audience, to, from, title, acid-test)`, `Helps(audience, reason, to: none)`, `Hinders(audience, reason, from: none)` and `Neutral(..goals)`. `Helps` and `Hinders` assert their label, so a missing `to:` or `from:` is a Typst error rather than a marker with no goal. `Neutral` emits one marker per goal.
- `truth/goals.typ`, `truth/process-goals.typ`: keep both registries and add one goal value per key, with the `to`, `from`, `title` and `acid-test` strings from `decide-the-goal-set-for-an-agnostic-engine.typ`. The key stays the current name until the migration. Each file wraps `goal` with its audience.
- `truth/goal-calls.typ`: re-export the DSL, both sets and every goal value under its own name. A task imports it with `*`, so `#Helps(adopter, to: read-tasks-at-a-glance)` names the value. The mechanism is internal.
- `truth/lib.typ`: import `goal-dsl.typ`, add `check-goal-over(markers)` beside `check-goals`, and keep `surfaced-by` and the 0.3.0 pin. The pin stays: 603 tasks import it. The migration narrows it to `edge`.
- `crates/mindtape-cli/tests/it/goal_markers_e2e.rs`: copy the real goal files and `truth/lib.typ`, and point the rule at `check-goal-over`. A missing `to:` or `from:` fails the load. A wrong audience, a duplicate, a missing or unknown goal, an empty reason and no helps breach. A marker-free task as `[]` breaches. A complete block, a second helps, a hinders with a reason and an unrelated marker pass. A dropped body emits none.
- `crates/mindtape-cli/tests/it/main.rs`: add `mod goal_markers_e2e;` between `go_e2e` and `hint_scope_e2e`.

== Out of scope

- The live rule switch, the registry rename and the backlog migration. The migration leaf. The DSL keeps the current keys. The examples show the post-migration names. The migration renames them and keeps the old names as aliases, so a task filed in the window still loads.
- Docs and AGENTS. The migration leaf.
- A goal rename after a task closes. A task-facing name is immutable. A rename keeps the old function as a one-line alias that forwards to the new key.

== Dogfood

The first task filed after the migration uses the DSL, and the check refuses a goal under the wrong audience.
