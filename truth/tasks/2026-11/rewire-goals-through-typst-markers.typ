#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Rewrite goals through Typst metadata markers",
  kind: "milestone",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("mt", "tooling", "gating", "stateless"),
  assignee: "agent",
  status: proposed(2026, 11, 21),
)

== AS-IS

+ 🔴 A proposed task's `x-goals` is a header dict of free-text keys and reasons. A goal is a string, so a misspelling is a check breach rather than a compile error, and a reason carries no markup.
+ Open `truth/lib.typ`. 🔴 `check-goals` reads the dict's strings. A goal is not a Typst reference.
+ Emit `#metadata(...)` in a task body. 🔴 `mt` harvests it, then keeps only `mindtape.task`. A check function cannot see the rest, and a warm cache would lose it again.

== TO-BE

- 🟢 A task body emits metadata markers. `Task` and the cache carry them, and a check with `field = "markers"` reads them as values.
- 🟢 The project defines `Helps`, `Hinders` and `Neutral`: `Helps` takes `to:`, `Hinders` takes `from:`, the reason is the trailing block, and `Neutral` takes the goals it rates neutral.
- 🟢 Every open task rates each goal once. `#Helps(adopter, to: <goal>)[reason]` and `#Hinders(maintainer, from: <goal>)[reason]` take a reason. `#Neutral(<goal>, ...)` takes none.
- 🟢 The goal set describes the engine boundary: `keep-the-engine-agnostic` replaces `config-over-code`, `run-checks-at-the-write` replaces `enforced-practice`, `beat-markdown` claims structured references.
- 🟢 Templates ask for one goals block at the end. Closed history keeps `x-goals`.

```typ
== Goals

+ #Helps(adopter, to: run-checks-at-the-write)[because a project's rule refuses the write].
+ #Hinders(maintainer, from: keep-changes-safe)[because a hand-merged marker fails the gate].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
```

== Changes

- `truth/tasks/2026-11/expose-task-body-markers-to-check-functions.typ`: the generic `BodyMarker` field, cache and `field = "markers"` input.
- `truth/tasks/2026-11/carry-body-markers-through-the-candidate-door.typ`: `mt add`, `import`, `link` and `recur` see the same markers a load does.
- `truth/tasks/2026-11/emit-project-imports-when-a-writer-rebuilds-a-task-file.typ`: `mt import` keeps the module imports a task body calls.
- `truth/tasks/2026-11/decide-the-goal-set-for-an-agnostic-engine.typ`: the goal names, the two spellings and the acid tests.
- `truth/tasks/2026-11/define-the-goal-dsl-over-metadata-markers.typ`: the goal values, `Helps`/`Hinders`/`Neutral` and the new check beside the old one.
- `truth/tasks/2026-11/migrate-the-open-backlog-and-switch-the-goal-rule.typ`: the registry rename, the live rule switch, the heading allowlist and every open task.
- `truth/tasks/2026-11/migrate-the-templates-and-the-scaffold-to-goal-markers.typ`: the `goals` slot, the multi-line default and the sweep arms.

Gate order: the exposure, emitter and decision leaves can run in parallel. The candidate door follows exposure. The DSL follows exposure and the decision. The backlog migration follows the DSL, the candidate door and the emitter. The templates follow the backlog migration, because a template's `goals` slot only passes once the rule reads markers.

== Out of scope

- The conflict-marker gate.
- A path-set warning that guesses an audience from `derived.changes`. It can false-positive on a task that touches both surfaces.
- Migrating closed tasks.
- Adding `consistent-tree`: it overlaps `keep-changes-safe` and `match-docs-to-code`, and every new goal re-migrates the open backlog.
- Moving the goal DSL into the prelude. Filed as `ship-the-goal-dsl-in-the-prelude.typ`. The DSL is this repo's own configuration, so an adopter is not hurt until the prelude carries it.
- Showing markers in `mt eval --json` or `mt ls --json`. The check input is the consumer. The repo has no consumer of goal verdicts from `mt eval --json` today. A future one has no feed until a later leaf. The task file is the source.

== Dogfood

This repo's own backlog is the first goal check. Every open task and active template uses the marker DSL once the two migration leaves land.

== Goals

+ #Helps(adopter, to: guide-the-implementer)[the milestone splits the loader, the candidate door, the emitter, the project DSL and the migration into leaves with a gate order].
+ #Helps(adopter, to: run-checks-at-the-write)[a project check reads structured markers the task body emits].
+ #Helps(adopter, to: keep-the-engine-agnostic)[the marker field is generic. The goal policy stays in project Typst].
+ #Helps(adopter, to: beat-markdown)[the marker field lets a rule read Typst values instead of parsed prose].
+ #Neutral(read-tasks-at-a-glance, adopt-in-one-line, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
