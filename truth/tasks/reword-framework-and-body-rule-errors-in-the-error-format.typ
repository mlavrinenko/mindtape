#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Reword framework and body rule errors in the error format",
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

+ Declare a priority framework whose score uses `%` and run `mt ls`.
+ 🔴 It prints `score uses unsupported operator Mod`, a Rust variant name.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.

== Changes

- `crates/mindtape/src/framework/eval.rs`: no `{:?}` (lines 143 and 151 today).
- `crates/mindtape/src/framework/validate.rs`, `crates/mindtape/src/body/headers.rs`, `crates/mindtape/src/body/gate.rs`, `crates/mindtape/src/model/numeric.rs`: quote values; the key path stays first.

== Out of scope

- `GateError`'s context in `pre_write.rs` and `check/mod.rs`: their own tasks.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
