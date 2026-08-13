#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "derive the formatter path lists from justfile assignments",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "tooling"),
  estimate: (files: 3),
  links: related(
    "derive-the-gate-table-command-column-from-the-justfile.typ",
  )[same table, same dump, same generator pass],
  status: proposed(2026, 8, 14),
)

== Summary

`just --dump --dump-format json` carries an `assignments` block with `TOML_FMT_PATHS`, `TYPST_FMT_PATHS` and `CRAP_EXCLUDES` verbatim. CONTRIBUTING.md describes those variables in prose; it can render their actual contents.

== Scope

- `www/generate-facts.sh` folds the three assignments into `gates.json` (or a sibling artifact) alongside the gate list.
- The `fmt-check` entry in the gate notes renders the real path lists instead of naming the variables and trusting the reader to go look.
- The rule the prose states — one list feeding both `fmt` and `fmt-check`, so a path is added in one place — stays prose. It is a convention, not a value.
- Keep it honest about size: if a rendered list is long enough to bloat the table, render it under the table rather than inside a cell.
