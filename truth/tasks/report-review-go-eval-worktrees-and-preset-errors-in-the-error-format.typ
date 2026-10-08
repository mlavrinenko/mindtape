#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report review, go, eval, worktrees and preset errors in the error format",
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
  status: wip(2026, 10, 8),
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== Problem

+ Run `mt go <task>` with no editor configured.
+ 🔴 It prints `Error: no editor configured: set `[go] editor` in .mindtape/config.toml (or $VISUAL / $EDITOR)`, the fix inside the error.
+ Run `mt @nope ls`.
+ 🔴 It prints `Error: unknown worktree @nope; known worktrees: mt, sn` and an indented second line.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.

== Changes

- `crates/mindtape-cli/src/cmd/review.rs`, `crates/mindtape-cli/src/cmd/go.rs`, `crates/mindtape-cli/src/cmd/eval.rs`, `crates/mindtape-cli/src/cmd/worktrees.rs`, `crates/mindtape-cli/src/cmd/preset.rs`: `Diag`s with hints; drop their allowlist lines in `crates/mindtape-cli/src/cmd/mod.rs`.

== Out of scope

- `mt link`: `report-set-and-link-errors-in-the-error-format`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
