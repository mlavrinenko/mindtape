#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Name the stale group and its command in outdatty-check",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 2,
  estimate: (src-files: 2),
  tags: ("found", "tooling"),
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
      sound-codebase: helps[a failing drift gate says which group to review and how to record it],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 7)[the recipe prints each stale group from the JSON],
)

== Problem

+ Merging four parallel docs rewrites
+ 🔴 #raw("Justfile"): a failing `just outdatty-check` prints only source paths, with no group and no command to run

```
$ just outdatty-check   # after editing two api-service sources
examples/api-service/.mindtape/config.toml
examples/api-service/scripts/test
error: Recipe `outdatty-check` failed on line 197 with exit code 1
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 A failing run names each stale group and the `just outdatty-update` command that records it.

== Changes

- `Justfile`: `outdatty-check` prints each stale group from `outdatty check --format json`, with what changed, what to review and the update command.
- Upstream, outdatty's plain `check` output could list only the groups that failed, and the recipe would call it directly.

== Rollback

- Revert the `Justfile` recipe.

== Out of scope

- The outdatty manifest groups themselves.
- Gating a dependent edited on its own. `outdatty.yaml` keeps that ungated on purpose, so a doc edit doesn't turn the gate red.

== Dogfood

Every commit here that edits a dependent records it, or the gate says how.
