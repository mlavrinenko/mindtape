#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report flip refusals and gate failures in the error format",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
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
  ),
)

== Problem

+ Run `mt done <task>` in `examples/api-service` with its `[[on.flip]]` gate failing.
+ 🔴 It prints `error: <task>: gate "scripts/test" failed (exit 1); nothing written — fix it, or re-run with --force (the override is recorded in the status note)`: one long line that names the task twice and says "nothing written", which `error:` already means.

== TO-BE

- 🟢 Each message is built with `Diag` (`crates/mindtape-cli/src/diag/`), so it follows the format in that module's docs: one `error:` line, the typed value single-quoted, the next step on a `hint:` line.
- 🟢 A failing gate prints `error: gate 'scripts/test' failed (exit 1)`, the config's `message` as a `hint:`, and `hint: or pass --force to skip the gate; the status note records it`.
- 🟢 `flip_target` returns a `Diag`, so `Diag::legacy` has no caller in `cmd/flip/`.

== Changes

- `crates/mindtape-cli/src/cmd/flip/hooks.rs`: the gate refusal as a `Diag` with the config message as a hint; `mt` adds the `--force` hint itself.
- `crates/mindtape-cli/src/cmd/flip/guards.rs`: the transition, note, blocker and children guards return a `Diag`.
- `crates/mindtape-cli/src/cmd/flip/batch.rs`: revalidation problems as `Diag`s, from the breach's own conversion.
- `crates/mindtape-cli/src/cmd/flip/mod.rs`: `flip_target` returns a `Diag`, so no `Diag::legacy` is left in `cmd/flip` and its `diag/allowlist.rs` entry goes; the shadowed-verb warning stays as it is.

== Out of scope

- The pre-write check's own lines: `report-pre-write-refusals-once-without-a-nested-summary`.
- The gate messages in `.mindtape/config.toml`: the rule-messages task.

== Dogfood

Every `mt` error in this repo's sessions from the files above comes out in the new shape.
