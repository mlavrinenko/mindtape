#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Derive the built-in priority tables into the docs",
  kind: "docs",
  status: proposed(2026, 9, 19),
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("www", "docs", "tooling"),
  estimate: (files: 6),
  assignee: "agent",
  x-goals: (
    product: (
      readable-tasks: helps[a table that cannot drift is worth more than a hand-typed one],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[framework inputs, score formulas and difficulty anchors come off the source that declares them],
    ),
  ),
)

== Problem

1. 🔴 `/priority/`'s framework table, its ICE description and its new
  difficulty anchors are hand-typed from `crates/mindtape/src/frameworks.toml`
  and `defaults.toml`. A framework added, renamed or re-ranged in either file
  ships docs that still describe the old one, and no gate reacts.

== TO-BE

- 🟢 `www/generate-facts.sh` emits a `frameworks.json` fact: each framework's
  name, input names and types, score formula where it has one, and the
  priority/difficulty ranges with their labelled anchors.
- 🟢 A `www/utils/framework-notes.typ` helper renders it; `/priority/` reads
  the table from there instead of typing inputs.

== Source of truth

- `crates/mindtape/src/frameworks.toml` and `defaults.toml` — the declarations.
- `www/generate-facts.sh` — the fact pattern the readme task already uses.

== Changes

- `www/generate-facts.sh`, a new util, `/priority/`, outdatty wiring.

== Out of scope

- The framework-specific scoring prose (what the formula MEANS stays prose);
  only names, inputs, ranges and formulas are derived.

== Dogfood

`just check-doc-facts` plus a render check that the page's table matches a
deliberately changed range in a scratch copy of the defaults.
