#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep the project name in the next-page hint of mt @name ls",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "cli"),
  x-goals: (
    product: (
      readable-tasks: helps[the hint a listing prints runs as printed],
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
  status: proposed(2026, 10, 17),
)

== Problem

+ Dogfooding milestone 4q2 from `~` with `mt @mt ls -n 2`.
+ 🔴 #raw("crates/mindtape-cli/src/cmd/ls/mod.rs"): A single-project `@name` turns into `-w <path>` before `ls` parses, so the next-page hint drops it. A set's hint keeps its `@name`.


```
$ cd ~ && mt @mt ls -n 2 | tail -1
3 more not shown · next page: mt ls -n 2 --skip 2
$ mt @home ls ready -n 3 | tail -1
27 more not shown · next page: mt @home ls 'ready' -n 3 --skip 3
```
`~/.cargo/bin/mt` 0.3.0 prints the same `mt ls -n 2 --skip 2`, so it predates 4q2.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/ls/mod.rs").

== TO-BE

- 🟢 `mt @mt ls -n 2` prints `next page: mt @mt ls -n 2 --skip 2`.
- 🟢 `mt -w ../x ls -n 2` prints a hint that keeps `-w ../x`.


== Changes

- `crates/mindtape-cli/tests/it/ls_set_e2e.rs`: the red test first, a single-project `@name` listing cut by `-n`.
- `crates/mindtape-cli/src/cmd/ls/mod.rs`: `next_command_hint` restates how the run selected its project, from the `@name` the desugar saw or from `-w`.


== Rollback

- Revert the commit; the hint drops the selector again.


== Out of scope

- Other hints that name an `mt` command, which each get their own finding if they drop the selector.


== Dogfood

The owner pages through `mt @sn ls -n 20` from any directory by pasting the hint.

