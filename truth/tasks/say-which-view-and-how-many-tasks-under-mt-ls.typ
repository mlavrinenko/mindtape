#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Say which view and how many tasks under mt ls",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: helps[the line under a listing says where it came from and how much of the project it shows],
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
  status: proposed(2026, 10, 15),
)

== Problem

+ Run `mt init` in an empty git repo, then `mt add Fix the login` and `mt add Write docs`.
+ Run `mt ls`.
+ 🔴 The line under the table reads `applied: default (filter)`, which names a facet nobody asked about and no count.
+ Run `mt ls '' -n 1`.
+ 🔴 It prints `applied: manual (filter)` and `truncated: 1 of 2 — next: mt ls '' -n 1 --skip 1`.


== Environment

- `mt` built from `main` at the commit that filed this, on Linux.


== TO-BE

- 🟢 Bare `mt ls` ends with `default view · 2 of 2 tasks`: the source, then the rows shown of the tasks in the project.
- 🟢 Each other source has the same shape: `your filter`, `your filter and sort`, `release rule`, `sort and columns from inbox view` when several sources share the listing, and `release rule overridden` when every facet of a matched rule lost.
- 🟢 A listing nothing shaped prints just the count, `2 of 2 tasks`.
- 🟢 A cut listing adds `1 more · next page: mt ls '' -n 1 --skip 1`.
- 🟢 `--json` keeps its envelope.


== Changes

- `crates/mindtape-cli/src/cmd/ls/rules_applied_ls_rules_tests_applied.rs`: the red tests first.
- `crates/mindtape-cli/src/cmd/ls/rules_applied.rs`: the footer from `Applied` and the counts.
- `crates/mindtape-cli/src/cmd/ls/mod.rs`: carry the project's task count, print the footer and the next-page line.
- `crates/mindtape-cli/src/render/output.rs`: drop `Truncation::footer`, which only a test called.
- `www/content/cli/ls.typ`, `docs/src/skill.typ`: quote the new line.


== Out of scope

- The `--json` envelope's keys. A program reads those, and the count is `tasks.length` there.


== Dogfood

Every `mt ls` in this repo ends with the new line.

