#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Evaluate board columns from at.toml in each project",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[the board's columns are filter expressions in one TOML file, evaluated over plain task files],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 17),
  links: parent("show-every-tracked-project-from-one-mt-name.typ")
    + depends-on("parse-at-toml-into-named-project-sets.typ"),
)

== Problem

+ Ask how many tasks are open, ready and blocked in each of 30 projects.
+ 🔴 Each count is a `mt -w <path> ls <filter> --format path | wc -l` per project, and a filter such as `status == "wip"` fails in a project whose legend lacks `wip`.


== TO-BE

- 🟢 `at.toml` holds `[board]`, and `[at.<name>.board]` overrides it for a set:
```toml
[board]
sort = ["ready:desc"]
hide = "open == 0"

[[board.columns]]
name = "wip"
filter = "@wip"
fallback = 'status == "wip"'
```
- 🟢 A `filter` cell counts the project's matching tasks under its own config, and `@view` uses the project's view, else `fallback`. A `top` cell is the first task of a listing.
- 🟢 A cell whose filter fails is an error naming it. `hide` and `sort` read cells by column name.
- 🟢 The shipped `board.toml` names no status.


== Changes

- `crates/mindtape/src/at/board_tests.rs`: the red tests first: counts, the view fallback, `top`, a failing cell, `hide`, `sort`, a set's override.
- `crates/mindtape/src/at/board.rs`: parse `[board]`, evaluate one project's row, then sort and hide rows.
- `crates/mindtape/src/at/board.toml`: the shipped columns, open, ready, blocked and next.
- `crates/mindtape/src/at/mod.rs`: hold the board per entry, and list the files.
- `crates/mindtape/src/lib.rs`: re-export the board types.


== Alternatives

- `[board.columns]` as a table: TOML keeps no table order, so the columns would sort by name.
- A status map per project, such as `wip = ["doing"]`: a second vocabulary to keep in step, where a view of the same name uses the filter language.
- Columns built into Rust: they would name statuses, which `AGENTS.md` forbids.


== Out of scope

- The `mt board` command and its table, which the next task adds.
- A last-activity column: it needs a `max()` aggregate the filter language lacks.


== Dogfood

The owner's board counts this repo with the shipped columns, beside the other projects under `~/projects/home`.

