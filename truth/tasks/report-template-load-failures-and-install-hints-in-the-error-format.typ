#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report template load failures and install hints in the error format",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 4),
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
    "report-typst-errors-at-their-line-with-typsts-hints.typ",
    "report-task-references-and-project-discovery-in-the-error-format.typ",
  ),
)

== Problem

+ Break a template so it no longer loads and run `mt check`.
+ 🔴 It prints `<template>: did not load: eval error: <message>`, not the `error: file:line:col:` line a task file gets.
+ Run `mt ls` in a project without the `@local/mindtape` package installed.
+ 🔴 It prints `note: ... is not installed — run ...`: a next step on a `note:` line, with an em dash.

== TO-BE

- 🟢 A template that fails to load prints as a task file does, through `Diag`: `error: .mindtape/templates/bug.typ:4:3: <Typst words>` with Typst hints as `hint:` lines.
- 🟢 The install and `#`-in-the-body advice prints as `hint:` lines.
- 🟢 `util.rs` no longer needs its `allow`, so the list in `diag/allowlist.rs` loses `util`.

== Changes

- `crates/mindtape-cli/src/cmd/check/templates.rs`: `did not load` lines as `Diag`s, as `util.rs` builds them after `report-typst-errors-at-their-line-with-typsts-hints`.
- `crates/mindtape-cli/src/cmd/install/hint.rs`: its notes become `Hint`s on the load error.
- `crates/mindtape-cli/src/main.rs`: drop the `allow` on `mod util`.
- `crates/mindtape-cli/src/diag/allowlist.rs`: drop the `util` entry; its test refuses the drop while `util.rs` still prints.

== Out of scope

- The rest of `cmd/check`: `report-check-problems-on-stderr-in-the-error-format` does it and drops the `check` entry.

== Dogfood

Every `mt` error in this repo's sessions about a broken template or a missing package comes out in the new shape.
