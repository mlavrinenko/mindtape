#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show one row per project with mt board and a bare mt @name",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: helps[one short command shows where every project stands, with no service behind it],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: wip(2026, 10, 17),
  links: parent("show-every-tracked-project-from-one-mt-name.typ")
    + depends-on(
      "evaluate-board-columns-from-at-toml-in-each-project.typ",
      "resolve-mt-name-through-at-toml-and-rename-mt-worktrees-to-mt-at.typ",
      "carry-the-project-root-and-hyperlink-choice-in-renderctx.typ",
      "load-every-project-of-a-set-in-parallel.typ",
    ),
)

== Problem

+ Run `mt @mt`.
+ 🔴 It prints `error: 'mt' requires a subcommand but one was not provided`, and no command shows where each project stands.


== TO-BE

- 🟢 `mt board` prints one row per project of the selection, or one for the current project. A bare `mt @home` runs `mt @home board`.
```
PROJECT     OPEN  READY  BLOCKED  NEXT
speconaut     11     11        0  a9m Retires slot + check: error on task…
mindtape       3      3        0  cv9 List the active rules in one line…
⚠ kimmable   .mindtape/config.toml:107:17: can't parse config

@home · 27 projects · 19 hidden · mt @home board --all
```
- 🟢 A failed cell prints `?` with a note under the table naming its project.
- 🟢 `--all` shows hidden rows, and `--json` prints the rows.


== Changes

- `crates/mindtape-cli/tests/it/board.rs`: the red tests first: rows, a hidden row, a broken member, a bare `mt @name`, `--json`.
- `crates/mindtape-cli/src/cmd/board.rs`: rows from `open_set` and the `mindtape::at` board, then the table, notes and footer.
- `crates/mindtape-cli/src/cmd/at.rs`: rewrite a bare `mt @name` to `mt @name board`.
- `crates/mindtape-cli/src/cli/mod.rs`, `crates/mindtape-cli/src/cmd/mod.rs`, `crates/mindtape-cli/src/cli/command_class.rs`: wire `board` as a read command.
- `www/content/cli/board.typ`: the generated reference page.


== Alternatives

- `mt overview`: long to type, while `mt @x` alone is free.
- A board whenever `mt` runs outside a project: bare `mt` would print help in one directory and a board in another.
- A bare `mt @x` as the only way in: the board would take no flags.


== Out of scope

- A TUI or web dashboard: `--json` is the seam for one.
- Opening a row's project: `mt @<project> ls` does that.


== Dogfood

The owner's `mt @home` replaces a shell loop over `~/projects/home`, with this repo as one of its rows.

