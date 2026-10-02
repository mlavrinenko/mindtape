#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse hand-typed code blocks in doc sources",
  kind: "dev",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("docs", "tooling", "www"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[a code block can only show what a checked fixture or a real run produced],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
)

== Problem

+ Count the fenced code blocks typed into `www/content`, `docs/src` and `www/utils`.
+ 🔴 About 120 fenced blocks are typed into the sources; `config.typ` alone has 27.
+ Run some of them. 🔴 `config.typ:736` (`kind = "spike"`) prints `unknown field kind`; `filters.typ:141` prints `unknown dict in filter: verbs` in a fresh project; the CSV recipe at `import-export.typ:143` fails on a framework priority.
+ `docs/contributing/documentation.md` already says a block must come from a checked fixture. Nothing checks it.


== TO-BE

- 🟢 `just check-doc-blocks` fails, naming file and line, on a fenced block or a `raw(..., block: true)` with a literal string in `www/content`, `www/utils` or `docs/src`.
- 🟢 A block that cannot come from a fixture, such as a grammar sketch, has `// docs:hand-typed <reason>` on the line above it.
- 🟢 A list in the script names the sources not yet converted. The gate fails when a listed file has no hand-typed block left, so the list only shrinks.


== Changes

- `.just/scripts/check-doc-blocks.sh`, new, with the ratchet list. Write the scan first and see it fail on today's tree; then seed the list.
- `Justfile`: the recipe, wired into `check`.
- `.mmz/config.yaml`, `outdatty.yaml`: scope and `dev-docs` entry.
- `www/utils/gate-notes.typ`: one clause naming the problem.
- `docs/src/contributing/documentation.typ`: the rule names the gate and the marker.


== Rollback

Revert the commit; drop the recipe from `check`.


== Out of scope

- Converting the listed pages: each page task removes its own files from the list.


== Dogfood

`just check` runs the gate on every commit; each docs task here shrinks the list.

