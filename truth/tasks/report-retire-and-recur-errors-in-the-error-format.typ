#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report retire and recur errors in the error format",
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

+ Run `mt recur <task>` on a task with no `recur` field.
+ 🔴 It prints `Error: "add-redis-client" has no `recur` field; nothing to advance`, the stem in Rust quotes.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.

== Changes

- `crates/mindtape-cli/src/cmd/rm/gitless/mod.rs`, `crates/mindtape-cli/src/cmd/rm/gitless/unlink.rs`: no em dash, I/O through the helper.
- `crates/mindtape-cli/src/cmd/recur/mod.rs`, `crates/mindtape-cli/src/cmd/recur/advance.rs`, `crates/mindtape-cli/src/cmd/recur/edit.rs`: `Diag`s; `stopping early` is a warning.
- `crates/mindtape-cli/src/cmd/mod.rs`: drop the `allow`s and their `diag/allowlist.rs` entries for `mv`, `rm` and `recur` once `report-mv-and-rm-errors-in-the-error-format` lands too.

== Out of scope

- `RecurError`'s words, already converted in `diag/from_lib.rs`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
