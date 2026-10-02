#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report every mt check breach with a severity and a file and line",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a reader of `mt check` finds every breach by file and line],
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
  status: proposed(2026, 10, 3),
)

== Problem

+ Running mt check on a hand-written task in examples/api-service to see what the docs captures will show
+ 🔴 #raw("crates/mindtape/src/gate.rs"): One mt check listing prints breaches in three shapes: a task rule with an error prefix and a file and line, a missing body section with neither, and a body content rule with a file and line but no prefix.

```
error: tasks/cache-the-product-list.typ:2: rules.split-big-work: split it into tasks of 5 source files or fewer
cache-the-product-list: body.headers.rules.one-to-be-section: every open task has one TO-BE section
tasks/cache-the-product-list.typ:8: body.content.rules.problem-is-numbered-steps: write the Problem as numbered steps anyone can follow
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/gate.rs").

== TO-BE

- 🟢 Every breach line in an `mt check` listing starts with its severity, then the file and line, so a reader scans one column.
- 🟢 A missing section reports at the task's `task.with(` line, the line a `[[tasks.rules]]` breach already uses.


== Changes

- `crates/mindtape/src/gate.rs`: when a diagnostic has no line, `diagnostic_to_violation` falls back to the task's `task.with(` line.
- `crates/mindtape-cli/src/check.rs`: `print_violation` prints a prefix on every breach. Its doc comment argues the prefix belongs to `[[tasks.rules]]` only, so the comment changes with it.


== Rollback

Revert the commit. The listing returns to three shapes.

== Out of scope

- The breach messages themselves.
- `mt add` refusals, which already start every line with `error:` and the stem.


== Dogfood

This repo's `mt check` and the docs captures of a refused task both print one shape.
