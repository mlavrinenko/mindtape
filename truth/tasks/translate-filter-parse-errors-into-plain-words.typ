#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Translate filter parse errors into plain words",
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

+ Run `mt ls 'foo('`.
+ 🔴 It prints `error: can't parse filter: 'foo(': Found an unmatched opening parenthesis '('`, evalexpr's capitalised words.
+ Run `mt ls '"wip"'`.
+ 🔴 It prints `unsupported in filter: Const { value: String("wip") }`, Rust's debug output.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 `mt ls 'foo('` prints `error: can't parse filter: 'foo(': a '(' is never closed`; `mt ls 'title ~ "x"'` says `'~' isn't an operator` with a `matches()` hint; no message shows a Rust type.

== Changes

- `crates/mindtape/src/filter/mod.rs`: map evalexpr's errors to `FilterError` variants with plain words; no `{:?}` of a node.
- `crates/mindtape/src/filter/expr.rs`: `expected a field` names the token as written.
- `crates/mindtape/src/filter/eval.rs`: `root needs one child` and `binary op needs two operands` become `the filter is incomplete`; `unknown function` gets a did-you-mean.
- `crates/mindtape/src/filter/compare.rs`, `crates/mindtape/src/filter/aggregate.rs`: the internal wording of the operand and `count()` messages.
- `crates/mindtape-cli/src/diag/from_lib.rs` gains a hint per new variant.

== Out of scope

- The CLI side, already converted for `mt ls` and `mt graph` in the error DSL task.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
