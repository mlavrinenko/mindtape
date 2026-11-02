#import "@local/mindtape:0.3.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Explain why a blocked task is blocked",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt", "cli"),
  estimate: (files: 4),
  x-goals: (
    product: verdicts(
      (
        implementable-tasks: helps[an agent reads what to unblock from one listing instead of walking links],
      ),
      "readable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    9,
    27,
  )[why column (named()), why list in --json, \@blocked view in this repo; documented on the CLI, schema and import pages and in the skill],
)

== AS-IS

+ `mt ls blocked`.
+ 🔴 It lists blocked tasks but not what blocks them; an agent must `mt eval` each task and then each target to learn which blockers are unfinished.


== TO-BE

- 🟢 A computed `why` column lists the unfinished blocking targets with their status:

```
$ mt ls blocked --columns title,why
#   TITLE                    WHY
1c  Publish 0.4.0            waits on: write-atomic (proposed), link-cycle (wip)
```

- 🟢 `--json` carries `why` as a list of `{stem, status, kind}` for every task, empty when ready.


== Changes

- `crates/mindtape/src/ready.rs` (tests first): return the unfinished blockers, not only the verdict.
- `crates/mindtape/src/canonical.rs`: the `why` list in `--json`.
- `crates/mindtape/src/config_ls_columns.rs`: the `why` column.
- `www/content/filters.typ`: document it.


== Alternatives

- A `--explain` flag as in `bd ready --explain`: a column composes with views and `--json` without a new flag.
- Transitive blockers: the one-hop list names what to act on; `blocks` already ranks leverage transitively.


== Out of scope

- Changing what counts as blocking: that is the link kinds' `blocking` flag.


== Dogfood

A `blocked` view in `.mindtape/config.toml` shows the `why` column.

