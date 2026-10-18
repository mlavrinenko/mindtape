#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move the board keys off the Read many projects page",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 1,
  tags: ("docs", "www"),
  estimate: (src-files: 2),
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
      docs-match-code: helps[the board keys live once, on the board page that the command links],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 18),
)

== Problem

+ Open `/read-many-projects/` and read "See every project at once".
+ 🔴 A start page carries the shipped board and two `at.toml` key tables, `board` and `board.columns`, that belong on `/cli/board/`.
+ 🔴 It spells out every status of shop and blog to say the board counts both right.


== TO-BE

- 🟢 The section says in two sentences which projects come first and how to change the board, linking `mt board`.
- 🟢 `/cli/board/` explains `[board]` and `[at.<name>.board]` and holds both key tables.
- 🟢 One sentence says each cell runs under its project's config, naming no status list.


== Source of truth

- `crates/mindtape/src/at/board.toml` and `schema/at.schema.json`: the shipped board and the keys.


== Changes

- `www/utils/cli-notes.typ`: the `board` note gains the override prose and the two key tables.
- `www/content/read-many-projects.typ`: the board section cut to its point.


== Out of scope

- The captures. They show the same runs.


== Dogfood

Not applicable. The page documents `at.toml`, which this repo doesn't use.

