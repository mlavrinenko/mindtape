#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the stale dead-code allowances on the set openers",
  kind: "dev",
  priority: ice(
    impact: 2,
    confidence: 0.8,
    ease: 9,
  ),
  difficulty: 1,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[an allowance no longer hides an unused item it was written for],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
)

== Problem

+ Marking the commands that run on a set in the command registry.
+ 🔴 #raw("crates/mindtape-cli/src/project_set.rs"): Two lint allowances still say a set opener waits for a later task, though `ls`, `find`, `check` and `board` all open sets now.

```
crates/mindtape-cli/src/project_set.rs:119:#[allow(dead_code, reason = "ls and board open a set in their own tasks")]
crates/mindtape-cli/src/util.rs:22:    reason = "ls, check and board open a set in their own tasks"
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/project_set.rs").

== TO-BE

- 🟢 No `#[allow]` on `open_set_warned` or the `util` re-export, or an `#[expect]` whose reason still holds.


== Changes

- `crates/mindtape-cli/src/project_set.rs`: drop the `dead_code` allowance on `open_set_warned`.
- `crates/mindtape-cli/src/util.rs`: drop the `unused_imports` allowance, or the re-exports nothing uses.


== Rollback

- Restore the two attributes.


== Out of scope

- Other allowances in the crate.


== Dogfood

Not applicable, a lint cleanup with no behavior.
