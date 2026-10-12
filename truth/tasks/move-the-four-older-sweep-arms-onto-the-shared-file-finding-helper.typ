#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Move the four older sweep arms onto the shared file_finding helper",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 4),
  tags: ("found", "tooling"),
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
      sound-codebase: helps[the file-or-print block lives in one place, so a fix to its messages lands once],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== Problem

+ Adding the repeats arm to `just ask-refactoring`, whose emit calls a new shared helper.
+ 🔴 #raw("contrib/ask-refactoring/size.sh"): The size, coverage, undertested and mutants arms each carry their own copy of the block that files a finding or prints it, which `file_finding` in `contrib/ask-refactoring.sh` now does once.

```
$ rg -c 'dedup skip --format json' contrib/ask-refactoring.sh contrib/ask-refactoring/
contrib/ask-refactoring.sh:1
contrib/ask-refactoring/size.sh:1
contrib/ask-refactoring/coverage.sh:1
contrib/ask-refactoring/undertested.sh:1
contrib/ask-refactoring/mutants.sh:1
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("contrib/ask-refactoring.sh").

== TO-BE

- 🟢 Every arm's emit builds its `mt` args and calls `file_finding`, and no arm runs `mt add --dedup skip` itself.


== Changes

- `contrib/ask-refactoring/size.sh`, `coverage.sh`, `undertested.sh`, `mutants.sh`: replace the block with a `file_finding` call, keeping each arm's dry-run summary.


== Rollback

- Revert the commit. Each arm gets its own block back.


== Out of scope

- The arms' scoring and templates.


== Dogfood

This repo's `just ask-refactoring` files through one helper.
