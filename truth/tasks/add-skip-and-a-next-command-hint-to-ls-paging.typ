#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add skip and a next-command hint to ls paging",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt", "cli"),
  estimate: (files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent with a small context pages through a backlog instead of guessing a filter],
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
  status: done(
    2026,
    9,
    27,
  )[skip advances -n paging; a truncated table's footer and --json's truncated.next\_skip name the exact --skip that continues; documented on the CLI, schema and agents pages and in the skill],
)

== Problem

+ `mt ls '' -n 20` in a large project.
+ 🔴 It prints the cut count but offers no way to the next page: there is no `--skip`, so an agent narrows the filter blindly or raises `-n`.


== TO-BE

- 🟢 `--skip N` drops N rows after the sort and before `-n`.
- 🟢 The table footer names the next command, and `--json`'s `truncated` carries `next_skip`:

```
$ mt ls -n 20
…
truncated: 20 of 143 — next: mt ls -n 20 --skip 20
```


== Changes

- `crates/mindtape-cli/src/ls_tests.rs` (red first), `crates/mindtape-cli/src/ls.rs`: `--skip` and the hint.
- `crates/mindtape/src/canonical.rs`: `next_skip` in the envelope.
- `www/content/cli.typ`: document it.


== Alternatives

- Cursor tokens: a live-evaluated tree has no stable snapshot for a cursor to point into; an offset over a deterministic sort is honest about that.


== Out of scope

- Stable paging while tasks change between calls: documented as a non-goal.


== Dogfood

Agents paging `truth/tasks/` through the skill.

