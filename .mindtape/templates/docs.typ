#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

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
  tags: slot("tags", kind: "list", default: ()),
  estimate: slot("estimate", kind: "text", default: "(src-files: 2)"),
  x-goals: (
    product: (
      readable-tasks: slot(
        "readable-tasks",
        kind: "raw",
        default: "neutral",
        example: "helps[a listing sorted by due shows what is due first]",
      ),
      implementable-tasks: slot(
        "implementable-tasks",
        kind: "raw",
        default: "neutral",
      ),
      enforced-practice: slot(
        "enforced-practice",
        kind: "raw",
        default: "neutral",
      ),
      cheap-adoption: slot("cheap-adoption", kind: "raw", default: "neutral"),
      typst-over-markdown: slot(
        "typst-over-markdown",
        kind: "raw",
        default: "neutral",
      ),
      plain-files: slot("plain-files", kind: "raw", default: "neutral"),
    ),
    process: (
      sound-codebase: slot("sound-codebase", kind: "raw", default: "neutral"),
      docs-match-code: slot(
        "docs-match-code",
        kind: "raw",
        default: "neutral",
      ),
    ),
  ),
)

== Problem

#slot(
  "problem",
  kind: "raw",
  prompt: "Numbered steps anyone can follow, commands in backticks; open the step that shows the defect with 🔴 and quote what it printed",
  example: ```typ
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
