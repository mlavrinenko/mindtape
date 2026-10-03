#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Report a write command's breach at the file and line of its field",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 4),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a refused `mt set` or flip points the editor at the offending line],
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

+ Closing the task that gave every \`mt check\` breach a file and line
+ 🔴 #raw("crates/mindtape-cli/src/pre_write.rs"): Write commands still print a validation breach under the task stem, where \`mt check\` now prints file and line.

`crates/mindtape-cli/src/flip_batch.rs:104`, `crates/mindtape-cli/src/task_edit.rs:137` and `crates/mindtape-cli/src/pre_write.rs:237` format `error: <stem>: field: message`; `README.md:54` and `README.md:78` show that shape.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/pre_write.rs").

== TO-BE

- 🟢 A breach that `mt set`, `mt flip` or `mt add` refuses prints `<file>:<line>: field: message`, the shape `mt check` prints.


== Changes

- `crates/mindtape-cli/src/pre_write.rs`: print each violation's location; a red test pins a refused `mt set` printing `file:line`.
- `crates/mindtape-cli/src/flip_batch.rs`: the same for a refused flip.
- `crates/mindtape-cli/src/task_edit.rs`: the same for a refused edit.
- `docs/src/readme.typ`: show the new shape.


== Rollback

Revert the commit; write commands print the stem again.

== Out of scope

- `mt check`, which already prints file and line.


== Dogfood

Agents here hit refused writes daily, and the error now opens at the line to fix.
