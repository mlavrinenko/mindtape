#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Open every AS-IS with one plain line that carries the essence",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("mt", "stateless"),
  assignee: "agent",
  status: proposed(2026, 11, 24),
)

== AS-IS

The template's `as-is` slot takes numbered steps alone, so an AS-IS opens on a command.

+ Read the `as-is` slot in `.mindtape/templates/task.typ`.
+ 🔴 Its prompt names numbered steps alone, and its example opens on `+ Run ...`, so no line says what is wrong.
+ Read the one open task, `truth/tasks/2026-10/decide-whether-jscpds-clone-classes-replace-the-repeats-arm-with-a-gate.typ`.
+ 🔴 Its AS-IS opens on `+ Read the jscpd-clone-classes blocker ...`, and the defect shows up only at the fourth step.

== TO-BE

- 🟢 Every open task's `AS-IS` opens with one plain line that carries the essence, before the numbered steps.
- 🟢 `mt check` refuses an `AS-IS` whose first line is a bullet, a marker or a fragment, and names the rule `open-as-is-with-the-essence`.
- 🟢 The template specifies the line first, and its example shows it.

```typ
== AS-IS

The repo has two duplicate checks: a gate at 50 tokens and an advisory at 15.

+ Run `just lint dry`.
+ 🔴 It reports nothing, while `just ask-refactoring` lists 26 snippets.
```

== Changes

- `crates/mindtape/src/packs/tasks.toml`: the `open-as-is-with-the-essence` rule, after `write-as-is-as-numbered-steps`.
- `.mindtape/templates/task.typ`: the `as-is` slot asks for the line first, and its example shows it.
- `.mindtape/templates/fix.typ`, `fix-stateful.typ`, `feat.typ`, `feat-stateful.typ`, `docs.typ`, `dev.typ`, `dev-stateful.typ`, `decision.typ`: the same `as-is` slot.
- `examples/api-service/.mindtape/templates/task.typ`: the same `as-is` slot.
- `examples/api-service/tasks/`: a summary line on each open task's `AS-IS`.
- `truth/tasks/2026-10/decide-whether-jscpds-clone-classes-replace-the-repeats-arm-with-a-gate.typ`: the summary line its `AS-IS` lacks.
- `www/content/templates.typ`, `www/content/dsl.typ`, `www/content/require-a-task-shape.typ`: the convention in the docs.
- `skills/mindtape/references/hand-editing.md`: the convention in the skill's reference.
- `docs/src/agents.typ`: the convention in AGENTS.md's source.
- `CHANGELOG.md`: the pack rule.

== Alternatives

- A rule on the first paragraph's shape alone: it cannot tell a summary from a stray sentence, so the line would pass without carrying anything.
- A `summary` slot of its own above `AS-IS`: one more slot to fill, and the reader still meets the steps first.

== Out of scope

- The repeats gate, the test-helper dedupe and the qahq bump. Each is its own task.

== Dogfood

This repo's own backlog is the first corpus the rule runs on: the one open task gets its line, and every task filed from here opens with one.

== Goals

+ #Helps(adopter, to: read-tasks-at-a-glance)[because the first line states the defect before any step runs].
+ #Helps(adopter, to: adopt-in-one-line)[because the rule lands in the pack every adopter extends].
+ #Neutral(guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
