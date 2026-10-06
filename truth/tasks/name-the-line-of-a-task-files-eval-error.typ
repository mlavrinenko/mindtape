#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the line of a task file's eval error",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer finds the broken line of a hand-edited task without bisecting the file],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Capturing \`mt check\` on broken task files for the /task-files/ docs page
+ 🔴 #raw("crates/mindtape/src/loader.rs"): an eval error in a task file names the file but not the line, so a syntax mistake in a 30-line task has to be found by eye

```
warning: tasks/missing-comma.typ: eval error: expected comma
error: tasks/one-tag-without-a-comma.typ:7: tags: expected an array, found a string
```
The rule violation names line 7, and the eval error names no line. typst-world's `format_diagnostics` (`src/world.rs:425`) keeps each diagnostic's message and drops its span.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("typst-world src/world.rs").

== TO-BE

- 🟢 `mt check` prints an eval error as `tasks/missing-comma.typ:5: eval error: expected comma`, with the line of the first diagnostic, as it does for a rule violation.


== Changes

- `typst-world` `src/world.rs` and `typst-harvest` `src/harvest.rs`: carry the first diagnostic's line beside the message in the eval error, and release both crates.
- `crates/mindtape` loader: put that line in the load error's location.


== Rollback

- Revert the loader change and pin the previous typst-harvest.


== Out of scope

- Rewording Typst's own messages.


== Dogfood

Every agent that hand-edits a task under `truth/tasks/` and breaks its syntax reads the line from `mt check` instead of searching the file.
