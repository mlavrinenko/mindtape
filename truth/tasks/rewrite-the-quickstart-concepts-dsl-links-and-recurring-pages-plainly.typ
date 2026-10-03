#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the quickstart, concepts, DSL, links and recurring pages plainly",
  kind: "docs",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer reads the model and the DSL without tripping on wrong claims],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[each claim on these pages matches the code],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
  links: depends-on(
    "write-plain-prose-rules-for-the-docs-and-check-for-history-and-shouting.typ",
    "refuse-hand-typed-code-blocks-in-doc-sources.typ",
    "print-each-captures-command-from-the-string-that-ran-it.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Read the pages against the code.
+ 🔴 `links.typ:16` and `concepts.typ:42` say a link target is a filename stem. `mt` resolves it as a path relative to the linking file (`crates/mindtape/src/model.rs:277`).
+ 🔴 `dsl.typ:64` says each status in the legend is a constructor. The prelude ships 11 fixed names (`lib/prelude.typ:52-62`); another status needs `status("name", y, m, d)`, which no page shows.
+ 🔴 `recurring.typ:84` says `mt check` parses the raw string either way; it returns early when a gate is closed (`recur.rs:395`). Shipped builds lack the `rrule` feature, and no page says how to enable it.
+ Live evaluation and the `x-<ns>` seam are explained on four or five pages each.


== TO-BE

- 🟢 Every claim matches the code. Concepts owns the model: live evaluation, identity with the `id:` override, path-based links, `x-<ns>`. Other pages link to it.
- 🟢 Concepts says `mt mv` rewrites inbound links on a rename.
- 🟢 /recurring/ says `cargo install mindtape-cli --features rrule`.
- 🟢 Code blocks are fixture slices or captures, and the prose follows the plain-prose rules.


== Source of truth

- `lib/prelude.typ`: fields, constructors, helpers.
- `crates/mindtape/src/model.rs`, `validate_links.rs`, `recur.rs`: link resolution and recurrence.
- `crates/mindtape-cli/Cargo.toml`: the `rrule` feature.


== Changes

- `www/content/{quickstart,concepts,dsl,links,recurring}.typ`: rewrite whole.
- `www/utils/dsl-field-notes.typ`: every constructor takes a date (drop "dated today"); "tuple" becomes "array"; fix "below" pointers to sections that moved.
- `examples/links`, `examples/recurring`: `docs:slice` markers for the blocks the pages need.
- `.just/scripts/check-doc-blocks.sh`: drop these pages from the list.


== Out of scope

- The quickstart's capture mechanism: the capture task builds it.


== Dogfood

This repo's own tasks use the links and recurrence these pages describe; `just check-examples` validates the slices.

