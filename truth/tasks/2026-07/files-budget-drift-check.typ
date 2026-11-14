#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "files budget drift check",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("cli", "gating"),
  status: done(2026, 7, 22),
  links: depends-on("required-as-filter-expression.typ"),
)

== Summary

Make a task declare how many files it intends to touch, and warn when the working tree outgrows that. The rationale comes from data on agent-authored pull requests. The unmerged ones touched about 10 percent more files and 17 percent more lines than the merged ones. Agent PRs bundled multiple objectives 39.9 percent of the time against 12.2 percent for humans.

A predicted duration is a guess a model will happily invent. A file count is predictable up front, is the same information a good task already carries as file guidance, and is checkable against git while the work runs.

The intent is not to punish an underestimate. It is to stop an agent tinkering onward inside a task that has outgrown itself, and to make splitting the obvious next move.

== Scope

- `[tasks.estimate]` gains `required`, using the filter-expression form from `required-as-filter-expression`.
- `[tasks.estimate]` gains per-unit `max` and `min` tables, such as `[tasks.estimate.max] files = 5`. Outside the range is a validation error at authoring time: split the task before work starts, not after.
- `mt check` compares the git working-tree change set against the in-flight estimate and warns on breach. Warning, never an error: an unforeseen file is normal, and the response is a rescope rather than a failure.
- The message is the whole intervention, so it is configurable. The default message spells out the action, namely split the remaining work into a new task and finish the part already in hand. A message that only reports the breach tells an agent nothing about what to do next.
- Attribution. The working tree carries one diff, and nothing in it says which file belongs to which task. So: exactly one task in flight, compare against that task and name it. With many in flight, compare against the sum of their budgets and say in the message that the figure is a total. This keeps the check useful without making `task-limits` a hard prerequisite.
- Line count is a secondary soft warning at a generous multiple of the declared file count, never a budget of its own. Generated files, lockfiles and formatting passes distort line totals.
- v1 compares counts only. The intended path list stays prose under the required Files heading, enforced by a body-header rule. Machine-comparing declared paths against the diff needs prose parsing and is out of scope.
- This project adopts it on its own backlog: `units = ["files"]`, a cap, and a required estimate. Sequencing is mandatory: land the feature, backfill every existing task, and only then make it required. Flip it first and the whole backlog fails validation at once.
- Tests: under, at and over the cap, and below `min`. The drift warning fires and stays a warning. One task in flight names it, and many in flight report a total. A terminal task is exempt.

== Notes

Closed on the feature and on the first half of the adoption: `units = ["files"]` and the `max`/`min` caps are on this repository's config. The `required` flip is not, and moved into `adopt-a-files-estimate-on-the-open-backlog.typ`.

The reason is the sequencing this task's own Scope calls mandatory. Requiring an estimate needs every open task to declare a files budget first. That backfill is a judgment call per task rather than a mechanical edit, exactly the work that deserves its own artifact instead of riding along in this one. Setting `required` without it would fail the whole backlog at once.
