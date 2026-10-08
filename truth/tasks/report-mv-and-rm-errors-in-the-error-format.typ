#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report mv and rm errors in the error format",
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

+ Run `mt mv a a/b`.
+ 🔴 It prints three clauses and two em dashes behind `Error:`.
+ Run `mt rm <task> </dev/null`.
+ 🔴 It prints `Continue? [y/N] Error: aborted by user`, though nobody answered.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 With stdin not a terminal, `mt rm` says it can't ask and hints `--yes`; a bare word that's no field and no stem says `no task`.

== Changes

- `crates/mindtape-cli/src/cmd/mv.rs`: one clause per message, the reason as a hint.
- `crates/mindtape-cli/src/cmd/rm/mod.rs`: `cannot remove` as error plus hint, the list as notes.
- `crates/mindtape-cli/src/cmd/rm/select.rs`: a non-matching ref says `no task`, not a filter error.
- `crates/mindtape-cli/src/cmd/rm/report.rs`, `crates/mindtape-cli/src/cmd/rm/gitless/report.rs`: decide whether the confirm prompt stays a bare stderr line, and write it through `crate::diag` either way.

== Out of scope

- The retire strategy and `mt recur`: `report-retire-and-recur-errors-in-the-error-format`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
