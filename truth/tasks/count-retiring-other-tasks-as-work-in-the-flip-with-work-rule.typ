#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Count retiring other tasks as work in the flip-with-work rule",
  kind: "dev",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  estimate: (src-files: 3),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a task whose work is retiring other tasks closes without padding its commit],
      cheap-adoption: helps[an adopter's cleanup task closes under the shipped pack without a workaround],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 14),
)

== Problem

+ Closing `accept-a-folder-path-in-mt-rm`, whose dogfood removes 250 done tasks with `mt rm truth/tasks/swept` in the closing commit
+ 🔴 #raw("crates/mindtape/src/packs/mindtape-tasks.toml"): the shipped `flip-with-work` rule reads a commit that retires other tasks plus the `done` flip as `tasks_only`, so it refuses a close whose work is retiring tasks, and the closer had to add an unrelated test to get past it

```
[[commits.rules]]
name = "flip-with-work"
scope = "task"
when = 'into("wip") || into("done")'
assert = 'named && (!tasks_only || staged(has_tag("decision")))'
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/packs/mindtape-tasks.toml").

== TO-BE

- 🟢 Any Mindtape user closes a cleanup task in the commit that runs `mt rm` on other tasks, with no extra file to make it count as work.
- 🟢 A commit holding only the flip, or the flip plus edits to other tasks' bodies, is still refused.


== Changes

- `crates/mindtape/src/commit/`: a commit predicate true when the staged tree retires a task other than the flipped one, if the language has none.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: `flip-with-work` accepts that predicate beside the `decision` escape hatch.
- `www/content/`: the page listing commit predicates and the shipped rules.


== Rollback

- Revert the commit; the rule goes back to refusing such a close.


== Out of scope

- `work-names-a-task`, which already passes a commit naming its task.


== Dogfood

The next sweep batch closes in the commit that runs `mt rm truth/tasks/swept`, with no padding.

