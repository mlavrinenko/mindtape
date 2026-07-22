#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "files budget drift check",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 6,
  tags: ("cli", "gating"),
  status: wip(2026, 7, 22),
  links: depends-on("required-as-filter-expression.typ"),
)

== Summary

Make a task declare how many files it intends to touch, and warn when the working tree outgrows that. The rationale is measured: across agent-authored pull requests the unmerged ones touched about 10 percent more files and 17 percent more lines than the merged ones, and agent PRs bundled several objectives 39.9 percent of the time against 12.2 percent for humans.

A predicted duration is a guess a model will happily invent. A file count is plannable up front, is the same information a good task already carries as file guidance, and is checkable against git while the work runs.

The intent is not to punish an underestimate. It is to stop an agent tinkering onward inside a task that has outgrown itself, and to make splitting the obvious next move.

== Scope

- `[tasks.estimate]` gains `required`, using the filter-expression form from `required-as-filter-expression`.
- `[tasks.estimate]` gains per-unit `max` and `min` tables, e.g. `[tasks.estimate.max] files = 5`. Outside the range is a validation error at authoring time: the task must be split before work starts, not after.
- `mt check` compares the git working-tree change set against the in-flight estimate and warns on breach. Warning, never an error — an unforeseen file is normal, and the response is a rescope rather than a failure.
- The message is the whole intervention, so it is configurable and its default names the action: split the remaining work into a new task and finish the part already in hand. A message that only reports the breach tells an agent nothing about what to do next.
- Attribution. The working tree carries one diff, and nothing in it says which file belongs to which task. So: exactly one task in flight, compare against that task and name it. Several in flight, compare against the sum of their budgets and say in the message that the figure is a total. This keeps the check useful without making `task-limits` a hard prerequisite.
- Line count is a secondary soft warning at a generous multiple of the declared file count, never a budget of its own — line totals are distorted by generated files, lockfiles and formatting passes.
- v1 compares counts only. The intended path list stays prose under the required Files heading, enforced by a body-header rule; machine-comparing declared paths against the diff needs prose parsing and is out of scope.
- MindTape adopts it on its own backlog: `units = ["files"]`, a cap, and a required estimate. Sequencing is mandatory — ship the feature, backfill every existing task, only then make it required, or the whole backlog fails validation at once.
- Tests: under, at and over the cap; below `min`; the drift warning fires and stays a warning; one task in flight names it; several in flight report a total; a terminal task is exempt.
