#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Reword the view, ls, hook and dict config errors in the error format",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[every error says what failed and what to try, in one shape],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== Problem

+ Put `[[on.flip]]` with a `title` on a `run` action in `.mindtape/config.toml` and run `mt ls`.
+ 🔴 It prints `` `title` does not apply to this action kind (title: list, gate/message: run) ``, a parenthetical nobody can act on.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 Every message here keeps its key path first and names the value in quotes; `ls_columns.rs:114` gains its key path.

== Changes

- `crates/mindtape/src/config/views.rs`, `crates/mindtape/src/config/ls.rs`, `crates/mindtape/src/config/ls_columns.rs`, `crates/mindtape/src/config/hooks.rs`, `crates/mindtape/src/config/dicts.rs`: quote values, drop em dashes and parentheticals, one clause each.

== Out of scope

- How the line reaches the user: `report-config-load-errors-at-their-file-and-line`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
