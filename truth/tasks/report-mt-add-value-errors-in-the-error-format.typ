#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report mt add value errors in the error format",
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

+ Run `mt add -f /nonexistent t`.
+ 🔴 It prints `error: read values from /nonexistent: No such file or directory (os error 2)`.
+ Run `mt add -t nope t`.
+ 🔴 It prints `Error: template "nope" not found under [".mindtape/templates"]`, a Rust debug list.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 `error: can't read values: '/nonexistent': no such file or directory`; unknown slots and templates get a did-you-mean hint.

== Changes

- `crates/mindtape-cli/src/cmd/add/mod.rs`: the `{err:#}\n{hint}` join becomes hints; the parked-task note is a `Note`.
- `crates/mindtape-cli/src/cmd/add/values.rs`, `crates/mindtape-cli/src/cmd/add/tokens.rs`: quote values, no `{:?}`.
- `crates/mindtape-cli/src/cmd/add/notes.rs`, `crates/mindtape-cli/src/cmd/add/hint.rs`: `note:` and `hint:` through `emit`.

== Out of scope

- The interactive and target paths: `report-mt-add-target-and-editor-errors-in-the-error-format`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
