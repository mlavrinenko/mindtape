#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Print the task count first in the mt ls footer",
  kind: "feat",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("cli", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

+ Run `mt ls` in this repo.
+ 🔴 The footer reads `default view (!terminal && (miss(start) || start <= now())) · 19 of 1076 tasks`. The count trails a filter that runs to the edge of the terminal.
+ Run `stty cols 60` and then `mt ls`.
+ 🔴 The line wraps at ` · ` and the count lands on the continuation line, under `  `, away from the table it counts.

```text
$ mt ls
...
default view (!terminal && (miss(start) || start <= now())) · 19 of 1076 tasks

$ stty cols 60 && mt ls
...
default view (!terminal && (miss(start) || start <= now()))
  · 19 of 1076 tasks
```


== TO-BE

- 🟢 The count leads the footer: `19 of 1076 tasks · default view (!terminal && (miss(start) || start <= now()))`.
- 🟢 The source parts keep their order after the count, and `1 broken` stays last.
- 🟢 A wrapped line keeps the count on its first line. A reader sees how many rows the table holds before the filter that chose them.
- 🟢 `mt @set ls` leads with its count too: `3 of 3 tasks in 2 projects · your filter`.
- 🟢 `mt ls --json` keeps its envelope, unchanged.

```text
$ mt ls
...
19 of 1076 tasks · default view (!terminal && (miss(start) || start <= now()))

$ stty cols 60 && mt ls
...
19 of 1076 tasks
  · default view (!terminal && (miss(start) || start <= now()))
```


== Changes

- `crates/mindtape-cli/src/cmd/ls/rules_applied_ls_rules_tests_applied.rs`: the red test first, the count leading the source parts.
- `crates/mindtape-cli/src/cmd/ls/rules_applied.rs`: `footer` pushes the count before `source_parts`.
- `crates/mindtape-cli/src/cmd/ls/set.rs`: the set footer leads with its count.
- `crates/mindtape-cli/tests/it/ls_limit_e2e.rs`: the expected footer.
- `crates/mindtape-cli/tests/it/find_e2e.rs`: the expected footer.
- `crates/mindtape-cli/tests/it/ls_broken_e2e.rs`: the expected footer.
- `crates/mindtape-cli/tests/it/ls_set_e2e.rs`: the expected footers.
- `www/content/cli/ls.typ`: the footer shape.
- `skills/mindtape/references/filters.md`: the footer example.
- `.config/outdatty.lock`: the `cli-ls-docs` re-record.


== Alternatives

- Leave the count last: the filter explains the rows and the count answers how many. It loses because the count is the shorter part and the one a reader looks for first.
- Put the count in the table header: the footer already holds every count, and a header row is per column.
- Move the count only when the line wraps: the order would then change with the terminal width.


== Out of scope

- The `next page` line and the `hint:` line. Each already leads with its own count.
- The `mt ls --json` envelope. It carries the facets, not the footer text.
- The `mt board` footer. It prints its own line.


== Dogfood

This repo's `mt ls` leads with `19 of 1076 tasks`. A session reads the backlog size before the default view's filter.


== Goals

+ #Helps(adopter, to: read-tasks-at-a-glance)[the count leads the line, and the backlog size reads before the filter that chose the rows].
+ #Neutral(guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, keep-changes-safe, match-docs-to-code).

