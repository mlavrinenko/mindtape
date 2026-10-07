#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse an unknown status value in a filter or a view",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a view or rule that tests a misspelled status fails loudly instead of matching nothing],
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
  status: proposed(2026, 10, 7),
)

== Problem

+ Capturing every error \`mt\` prints for the error inventory
+ 🔴 #raw("crates/mindtape/src/filter/mod.rs"): a status value that no \`\[tasks.status\]\` entry declares passes a filter and a view's \`filter\` without a word, and the filter matches nothing

```
$ mt ls 'status == "wpi"'
[exit 0]

# [views.broken] filter = "status == wip" appended to .mindtape/config.toml
$ mt ls
[exit 0]

$ mt ls 'status == wip'
Error: unknown field in filter: wip
[exit 1]
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/filter/mod.rs").

== TO-BE

- 🟢 `mt ls 'status == "wpi"'` fails, naming `wpi` and suggesting `wip`.
- 🟢 A view whose filter names an undeclared status fails at config load, naming the view.


== Changes

- `crates/mindtape/src/filter/mod.rs`: check a string compared with `status` against the legend.
- `crates/mindtape/src/filter/compare.rs`: pass the legend to the comparison check.
- `crates/mindtape/src/config/views.rs`: run the same check on each view's filter at load.


== Rollback

- Revert the three files; an unknown status matches nothing again.


== Out of scope

- Other enum fields such as `kind` and tags; one field first, the rest follow the same seam.


== Dogfood

This repo's views and rules that test `status` are checked against its legend on every `mt` run.
