#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add the todo.txt fixtures to the import recipe",
  kind: "docs",
  status: done(2026, 9, 22),
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 1,
  tags: ("www", "docs", "tooling"),
  estimate: (files: 5),
  assignee: "agent",
  x-goals: (
    product: (
      readable-tasks: helps[a recipe with a golden pair is one a reader can check],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
)

== Problem

1. 🔴 `/import-export/`'s todo.txt recipe is the only import path with no
  golden fixture. `contrib/jq/fixtures/` pairs an input and an output for
  Taskwarrior and GitHub Issues; a reader of the todo.txt recipe has nothing to
  compare against, and a change to the filter would go unnoticed.

== TO-BE

- 🟢 `contrib/jq/todo.txt.jq`, `fixtures/todo.txt.in.json` and
  `fixtures/todo.txt.out.json` exist, mirroring the Taskwarrior pair.
- 🟢 `/import-export/`'s recipe references the filter, and the fixture pair
  verifies the same way the others do.

== Source of truth

- `contrib/jq/taskwarrior.jq` and its fixture pair — the shape to mirror.
- `www/content/import-export.typ` — the recipe text.

== Changes

- `contrib/jq/todo.txt.jq` — the filter.
- `contrib/jq/fixtures/todo.txt.in.json` and `todo.txt.out.json` — the pair.
- `www/content/import-export.typ` — the recipe line.

== Out of scope

- Any runner for the fixtures; they are reader-facing data, as the existing
  pairs are.

== Dogfood

Run the filter over the input, diff against the output, and `mt import` the
output against `examples/minimal` in a scratch copy.
