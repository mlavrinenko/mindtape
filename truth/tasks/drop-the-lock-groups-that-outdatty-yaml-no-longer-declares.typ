#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the lock groups that outdatty.yaml no longer declares",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
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
      docs-match-code: helps[a hash in the lock always belongs to a group someone can review],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 17),
)

== Problem

+ Rebasing a branch that re-recorded cli-docs onto the split, in a scratch clone
+ 🔴 #raw("outdatty.lock"): `outdatty check` passes with lock groups the manifest doesn't declare, and the lock merge driver keeps a dropped group's leaves that the other branch changed, so a split or a rename leaves orphan hashes behind.


```
$ comm -23 <(lock groups) <(outdatty.yaml groups)
sweep-dedup
worktree-release-docs
$ git rebase main-sim   # a branch that re-recorded cli-docs
$ grep -A2 '^  cli-docs' outdatty.lock
  cli-docs:
    source:
      crates/mindtape-cli/src/cmd/ls/mod.rs: 3f4a9255…
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw(".just/scripts/outdatty-lock-merge.sh").

== TO-BE

- 🟢 `outdatty.lock` holds no group that `outdatty.yaml` doesn't declare.
- 🟢 A rebase over a split drops the old group's leaves instead of keeping them.


== Changes

- `.just/scripts/outdatty-lock-merge.sh`: drop each group the merged `outdatty.yaml` doesn't declare.
- `outdatty.lock`: remove `sweep-dedup` and `worktree-release-docs`.


== Rollback

- Revert the script and restore the two lock entries.


== Out of scope

- The orphan check itself belongs upstream in outdatty, whose `check` could report an undeclared lock group the way it reports an unrecorded dependent. File it there.


== Dogfood

The next group rename in this repo rebases with no orphan left in `outdatty.lock`.

