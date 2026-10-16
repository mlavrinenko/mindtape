#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print nothing for an empty path listing in mt ls",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
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
      sound-codebase: helps[the printed output matches what the render test pins],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 16),
)

== Problem

+ Counting tasks across projects with `mt ls <filter> --format path | wc -l` while prototyping a cross-project overview
+ 🔴 #raw("crates/mindtape-cli/src/cmd/ls/mod.rs"): `mt ls` with `--format path` prints a lone newline when no task matches, because `println!` adds one after the empty body

```
$ mt ls blocked --format path | od -c
0000000  \n
0000001
```
So `wc -l` counts 1 for an empty result, and a `while read` loop runs once with an empty path. `path_format_is_empty_for_no_tasks` in `crates/mindtape-cli/src/render/output_tests.rs` passes because it tests `render`, not what `mt ls` prints.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/ls/mod.rs").

== TO-BE

- 🟢 `mt ls <filter> --format path` prints nothing when no task matches, so `wc -l` says 0.
- 🟢 A non-empty listing still ends with a newline.


== Changes

- `crates/mindtape-cli/src/cmd/ls/mod.rs`: print no newline after an empty `path` body.
- `crates/mindtape-cli/tests/it/`: pin the empty output through the binary.


== Rollback

- Revert the `cmd/ls/mod.rs` change.


== Out of scope

- `path0`, which already prints nothing for no tasks.


== Dogfood

Scripts in this repo that count `mt ls --format path` lines get the right count.

