#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Restore the in_dict suggestion in commit-rule expression problems",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 3),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[an unknown dict reads the same in a commit rule as in a filter],
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
  status: proposed(2026, 10, 8),
)

== Problem

+ Dropping the did-you-mean clause when commit 2 moved suggestions to Unknown data
+ 🔴 #raw("crates/mindtape/src/commit/expr.rs"): commit\_expr\_problem and task\_expr\_problem return plain strings, so an unknown in\_dict in a commits.rules expression loses its did-you-mean and choices hints

```
pub(crate) fn commit_expr_problem(source: &str, config: &Config) -> Option<String>
pub(crate) fn task_expr_problem(source: &str, config: &Config) -> Option<String>
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/commit/expr.rs").

== TO-BE

- 🟢 An unknown dict in a commit rule prints hint lines with the nearest dict and the dicts to choose from.

== Changes

- `crates/mindtape/src/commit/expr.rs`, `crates/mindtape/src/commit/task_expr.rs`: return Problem, not String.
- `crates/mindtape/src/config/commits.rs`: prefix with Problem::under.

== Rollback

- Revert the commit.

== Out of scope

- Other expression kinds.

== Dogfood

Not applicable: no commit rule here uses in_dict.
