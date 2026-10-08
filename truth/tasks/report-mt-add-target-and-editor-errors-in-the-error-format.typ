#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report mt add target and editor errors in the error format",
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

+ Run `mt add t --edit </dev/null` with `EDITOR=false`.
+ 🔴 It prints `Error: editor "false" exited without saving`, the command in Rust quotes.
+ Run `mt add` with no slot values.
+ 🔴 It prints `Error: missing required slot(s): title, …` with no next step.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 A missing slot names `--set name=value` and `mt add --describe` in hints.

== Changes

- `crates/mindtape-cli/src/cmd/add/interactive.rs`, `crates/mindtape-cli/src/cmd/add/target.rs`, `crates/mindtape-cli/src/cmd/add/dedup.rs`, `crates/mindtape-cli/src/cmd/add/yaml_values.rs`: `Diag`s, no `{:?}`.
- `crates/mindtape/src/template/fill/mod.rs`: return the missing slot names, not a sentence.
- `crates/mindtape-cli/src/cmd/mod.rs`: drop the `allow` and its `diag/allowlist.rs` entry for `add`.

== Out of scope

- The value parsing: `report-mt-add-value-errors-in-the-error-format`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
