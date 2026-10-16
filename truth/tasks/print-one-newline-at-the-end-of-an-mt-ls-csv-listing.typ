#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Print one newline at the end of an mt ls csv listing",
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
      sound-codebase: helps[the printed csv matches what the render test pins],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 16),
)

== Problem

+ Fixing the lone newline of an empty `--format path` listing in `mt ls`
+ 🔴 #raw("crates/mindtape-cli/src/cmd/ls/mod.rs"): `mt ls --format csv` prints a blank line after the last record, because the csv writer already ends each record with a newline and `println!` adds a second

```
$ mt ls 'title == "zz"' --format csv | od -c
0000000   i   d   ,   t   i   t   l   e  \n  \n
0000012
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape-cli/src/cmd/ls/mod.rs").

== TO-BE

- 🟢 `mt ls --format csv` ends its output with exactly one newline, for an empty and a non-empty listing alike.

== Changes

- `crates/mindtape-cli/src/cmd/ls/mod.rs`: `print_body` prints a body ending in a newline or NUL as is and adds one newline to any other, so no format needs its own case. The table footer prints its own separating blank line, which the table body's trailing newline and `println!` used to supply.
- `crates/mindtape-cli/tests/it/ls_newline_e2e.rs`: pin the last byte of every format, and the table's blank line before its footer, through the binary.

== Rollback

- Revert the `cmd/ls/mod.rs` change.

== Out of scope

- The `path` and `path0` formats, which `fix(ls): print nothing for an empty path listing` settled.

== Dogfood

Scripts in this repo that parse `mt ls --format csv` stop seeing an empty last row.
