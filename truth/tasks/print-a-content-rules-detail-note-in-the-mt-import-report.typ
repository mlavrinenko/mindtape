#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print a content rule's detail note in the mt import report",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[an import refused by a content rule says where the rule matched, as mt check does],
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
  status: done(2026, 10, 25),
)

== Problem

+ Importing a task whose TO-BE breaks a content rule that sets a `message`
+ 🔴 #raw("crates/mindtape-cli/src/pre_write/mod.rs"): `Verdict::problems` renders each breach as one line, so the report drops the `note:` line `mt check` prints under the message

```
$ mt import --dry-run tasks.json
3 would file, 1 skipped
  fix-login-bug: skipped
    - <new task>:15: body.content.rules.no-open-questions: ask the user and write the answer in, since a task leaves no question open
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/pre_write/mod.rs").

== TO-BE

- 🟢 A skipped record lists each breach with its detail indented under it, the same words `mt check` prints as a note.
- 🟢 A breach with no custom message prints one line, as today.

== Changes

- `crates/mindtape-cli/src/pre_write/mod.rs`: carry the violation detail with each problem `Verdict::problems` returns.
- `crates/mindtape-cli/src/cmd/import/mod.rs`: print the detail under its problem line in the per-record report.

== Rollback

- Revert both files. The report goes back to one line per breach.

== Out of scope

- The report layout and the summary lines. Only the missing detail is in scope.

== Dogfood

Every `mt import` in this repo that a content rule refuses names the matching sentence.
