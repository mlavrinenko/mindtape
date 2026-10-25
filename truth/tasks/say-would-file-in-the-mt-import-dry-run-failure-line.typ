#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Say would file in the mt import dry-run failure line",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: helps[a dry run that reports what it would do never claims it wrote tasks],
      implementable-tasks: neutral,
      enforced-practice: neutral,
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
  status: proposed(2026, 10, 25),
)

== Problem

+ Importing a batch with `--dry-run` where one record breaks a rule
+ 🔴 #raw("crates/mindtape-cli/src/cmd/import/mod.rs"): `fail_on_skips` always builds `Diag::batch("filed", …)`, so a dry run that wrote nothing ends by saying it filed tasks

```
$ mt import --dry-run tasks.json
3 would file, 1 skipped
  …
dry run: nothing written
error: filed 3 of 4 tasks; 1 failed
[exit 1]
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/import/mod.rs").

== TO-BE

- 🟢 A dry run with a skipped record ends with `error: would file 3 of 4 tasks; 1 failed` and exits 1.
- 🟢 A real import keeps `error: filed 3 of 4 tasks; 1 failed`.

== Changes

- `crates/mindtape-cli/src/cmd/import/mod.rs`: pass the dry-run flag to `fail_on_skips` and pick the verb the summary line already uses.

== Rollback

- Revert the file. The dry run says filed again.

== Out of scope

- The exit code. A dry run that would skip a record still exits 1, so a script can test a batch before it writes.

== Dogfood

Not applicable. This repo imports no tasks in its gates, so only a person or an agent previewing an import sees the line.
