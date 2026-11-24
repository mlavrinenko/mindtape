#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: slot(
    "title",
    prompt: "Task title, imperative verb first",
    example: "Document where `--sort` puts a task without the field",
  ),
  kind: "docs",
  priority: ice(
    impact: slot("impact", kind: "int", default: 5),
    confidence: slot("confidence", kind: "float", default: 0.8),
    ease: slot("ease", kind: "int", default: 5),
  ),
  difficulty: slot("difficulty", kind: "int", default: 3),
  tags: slot("tags", kind: "list", default: ("stateless",)),
)

== AS-IS

#slot(
  "as-is",
  kind: "raw",
  prompt: "One plain line that carries the essence first, then numbered steps anyone can follow, commands in backticks; open the step that shows the defect with 🔴 and quote what it printed",
  example: ```typ
  A task without a `due` sorts first, above the one due tomorrow.

  + Run `mt ls --sort due:asc` in a project where some tasks have no `due`.
  + 🔴 The undated tasks print first, above the one due tomorrow.
  ```,
)

== TO-BE

#slot(
  "to-be",
  kind: "raw",
  prompt: "Bullets opening with 🟢: how it should work, behave and feel once this lands",
  example: ```typ
  - 🟢 A task without a `due` sorts after every dated task, in either direction.
  ```,
)

== Source of truth

#slot(
  "source-of-truth",
  kind: "raw",
  prompt: "Bullets: the code, config or live output the prose must match",
  example: ```typ
  - `crates/mindtape/src/sort.rs`: where a missing field sorts.
  ```,
)

== Changes

#slot(
  "changes",
  kind: "raw",
  prompt: "One bullet per file: `path`, then what changes there",
  example: ```typ
  - `crates/mindtape/src/sort_tests.rs`: the red test first, a missing `due` sorting last.
  - `crates/mindtape/src/sort.rs`: order a missing `due` after every present one.
  ```,
)

== Out of scope

#slot(
  "out-of-scope",
  kind: "raw",
  prompt: "Bullets: what not to touch, and why",
  example: ```typ
  - The other date fields. Each gets its own task once someone asks for it.
  ```,
)

== Dogfood

#slot(
  "dogfood",
  kind: "raw",
  prompt: "How this repo will use the change, or Not applicable and why",
  example: ```typ
  This repo's `mt ls --sort due:asc` lists the dated backlog first.
  ```,
)

#slot(
  "goals",
  kind: "raw",
  required: true,
  prompt: "Under `== Goals`, rate every goal: one `#Helps` with a reason, then `#Neutral` for the rest",
  example: ```typ
  == Goals

  + #Helps(adopter, to: guide-the-implementer)[because one template slot carries the whole goal block].
  + #Neutral(read-tasks-at-a-glance, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).
  ```,
)
