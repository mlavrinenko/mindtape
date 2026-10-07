#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fail outdatty-check on a dependent edited without a recorded hash",
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
      sound-codebase: neutral,
      docs-match-code: helps[the rule to record an edited dependent holds without a reviewer remembering it],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 7),
)

== Problem

+ Merging four parallel docs rewrites
+ 🔴 #raw("Justfile"): just outdatty-check passes when a dependent changed and its lock hash did not, and a failing run prints only source paths, with no group and no command to run

```
$ echo "" >> www/content/cli.typ && just outdatty-check; echo "exit=$?"
exit=0
$ just outdatty-check   # after editing two api-service sources
examples/api-service/.mindtape/config.toml
examples/api-service/scripts/test
error: Recipe `outdatty-check` failed on line 197 with exit code 1
```

This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("Justfile").

== TO-BE

- 🟢 `just outdatty-check` fails on a dependent whose hash differs from the lock, naming the group.
- 🟢 A failing run names each stale group and the `just outdatty-update` command that records it.

== Changes

- `Justfile`: `outdatty-check` also compares dependent hashes (via `outdatty status --format json`) and prints group names with the update command.
- The upstream fix belongs in outdatty itself: a `check` mode that covers dependents and a plain output naming groups.

== Rollback

- Revert the `Justfile` recipe.

== Out of scope

- The outdatty manifest groups themselves.

== Dogfood

Every commit here that edits a dependent records it, or the gate says how.
