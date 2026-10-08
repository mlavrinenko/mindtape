#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report pre-write refusals once, without a nested summary",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("cli",),
  estimate: (src-files: 3),
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
  status: done(
    2026,
    10,
    8,
  )[A refusal is its last breach with no summary; a breach on another task is one note per batch.],
  links: depends-on(
    "build-an-error-dsl-that-fixes-the-message-format-and-the-exit-code.typ",
  ),
)

== Problem

+ Run `mt wip add-redis-client index-the-orders-by-customer` in `examples/api-service`.
+ 🔴 Each target prints its `rules.one-wip` breach, the same breach on `reject-expired-sessions-on-token-refresh` with `(reported on …)` once per target, then `error: add-redis-client: 1 task(s) rejected; nothing was written` for each, then the batch summary.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 A breach the rule raises on another task is one `note:` per batch, and a refused write prints its breaches and no `N task(s) rejected` line.

== Changes

- `crates/mindtape-cli/src/pre_write.rs`: emit breaches and load failures as `Diag`s; return the refusal without a summary of its own.
- `crates/mindtape/src/validate/candidate.rs`: drop `(reported on <id>)`; give the caller the other task's breach so it can print it once as a note.
- `crates/mindtape-cli/src/task_edit.rs`: the same for single-task writes (`mt set`, `mt link`); drop the `validation issue(s)` summary.
- `crates/mindtape-cli/src/main.rs`: drop the `allow`s and their `diag/allowlist.rs` entries for `pre_write` and `task_edit`.

== Out of scope

- The flip batch summary, already in the error DSL task.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
