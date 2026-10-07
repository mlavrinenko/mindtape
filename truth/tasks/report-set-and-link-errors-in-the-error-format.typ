#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report set and link errors in the error format",
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

+ Run `mt set <task> titel=x`.
+ 🔴 It prints `Error: unknown field: titel; did you mean: title?; writable fields: …`.
+ Run `mt link <task> depends-on nope`.
+ 🔴 It prints `` error: <task>: `depends-on` edge target `nope.typ` names no task `` and `Error: 1 task(s) had problems`.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 `error: unknown field: 'titel'` with `hint: did you mean title?`; `mt link` names the target as typed and ends a batch like `mt flip`.

== Changes

- `crates/mindtape-cli/src/cmd/set.rs`: `Diag`s; the `status` refusal says to use `mt flip`.
- `crates/mindtape-cli/src/cmd/link/mod.rs`: the batch summary from `Diag::batch`.
- `crates/mindtape-cli/src/cmd/link/plan.rs`, `crates/mindtape-cli/src/cmd/link/edit.rs`, `crates/mindtape-cli/src/cmd/link/input.rs`: the target as typed, one sentence per message; drop the `allow`s and their `diag/allowlist.rs` entries for `set` and `link` in `crates/mindtape-cli/src/cmd/mod.rs`.

== Out of scope

- The pre-write refusal they share: `report-pre-write-refusals-once-without-a-nested-summary`.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
