#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move the known-name lists in config errors onto a hint line",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[every message follows the error format],
      cheap-adoption: neutral,
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
)

== Problem

+ Reading config errors against the error format in diag/mod.rs
+ 🔴 #raw("crates/mindtape/src/config"): config errors end with a known-name list on the error line, where the format puts the next step on a hint line

views.{name}.columns: unknown column: {}{}; columns: {legal_cols}


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/diag/from_lib.rs").

== TO-BE

- 🟢 A config error shows its list of valid names as a `hint: use one of: a, b` line.

== Changes

- `crates/mindtape-cli/src/diag/from_lib.rs`: split the list clause off the error line into a hint.

== Rollback

- Revert the commit.

== Out of scope

- The cmd/ modules other tasks convert.

== Dogfood

A config typo shows the valid names under the error.
