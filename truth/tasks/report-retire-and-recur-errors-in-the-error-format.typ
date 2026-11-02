#import "@local/mindtape:0.3.0": *
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
    product: verdicts(
      (
        cheap-adoption: helps[every error says what failed and what to try, in one shape],
      ),
      "readable-tasks",
      "implementable-tasks",
      "enforced-practice",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 8),
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== AS-IS

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
