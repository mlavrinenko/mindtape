#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Reword the task, pack, actor, status and link config errors in the error format",
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
  status: done(2026, 10, 8),
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== Problem

+ Put `extends = ["nope"]` in `.mindtape/config.toml` and run `mt ls`.
+ 🔴 It prints `extends: unknown pack `nope`; available packs: …`, the list folded into the error line behind `Error: load …` and `Caused by:`.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.

== Changes

- `crates/mindtape/src/config/tasks.rs`, `crates/mindtape/src/config/packs.rs`, `crates/mindtape/src/config/actors.rs`, `crates/mindtape/src/config/status.rs`, `crates/mindtape/src/config/links_validate.rs`: quote values, name a near match as a separate clause the CLI turns into a hint.

== Out of scope

- How the line reaches the user: `report-config-load-errors-at-their-file-and-line`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
