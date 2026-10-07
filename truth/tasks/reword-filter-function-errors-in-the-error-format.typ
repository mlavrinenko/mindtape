#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Reword filter function errors in the error format",
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

+ Run `mt ls 'matches(title)'`.
+ 🔴 It prints `Error: invalid filter: matches() takes two arguments: matches(field, "regex")`, the usage example folded into the error.
+ Run `mt ls 'word(title, 0) == "a"'`.
+ 🔴 It prints `expected a field, got FunctionIdentifier { identifier: "word" }`.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 A function's usage example is a `hint:` line, and no message carries an em dash or a Rust type.

== Changes

- `crates/mindtape/src/filter/strings.rs`: drop em dashes and `{:?}`; usage examples become hints.
- `crates/mindtape/src/filter/predicates.rs`: the same for `has()`, `changed()`, `touches()`, `exists`, `contains()`.
- `crates/mindtape/src/filter/condition.rs`: `ConditionError` gets the same words, and a `From` in `diag/from_lib.rs`.
- `crates/mindtape/src/commit/expr.rs`, `crates/mindtape/src/commit/task_expr.rs`: the commit-rule expression messages.

== Out of scope

- Parse errors: `translate-filter-parse-errors-into-plain-words`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
