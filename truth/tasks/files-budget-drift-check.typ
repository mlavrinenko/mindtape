#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "files budget drift check",
  priority: ice(
    impact: 8,
    confidence: 0.7,
    ease: 4,
  ),
  tags: (),
  status: proposed(2026, 7, 22),
)

== Summary

Make a task declare how many files it intends to touch, and warn when the working tree outgrows that. Rationale is measured, not stylistic: across agent-authored PRs the unmerged ones touched about 10 percent more files and 17 percent more lines than the merged ones, and agent PRs bundled several objectives 39.9 percent of the time against 12.2 percent for humans. A predicted duration is a guess a model will happily invent; a file count is plannable up front, is the same information a good task already carries as file guidance, and is checkable against git mid-run. This is the mechanism that turns rescope-early from advice into a signal.

== Scope

- `[tasks.estimate]` gains `required` (true | false | "non-terminal"), mirroring the existing `[tasks.priority] required`.
- `[tasks.estimate]` gains a per-unit cap table, e.g. `[tasks.estimate.max] files = 5`. Over the cap is a validation error at authoring time: the task must be split before work starts.
- `mt check` compares the git working-tree change set against the in-flight task estimate and warns on breach. Warning only, never an error: an unforeseen file is normal and the correct response is rescope, not failure.
- Attribution rule: the comparison runs only when exactly one task matches the configured in-flight view. Zero or several, skip the comparison and say why. This is why task-limits is a hard prerequisite.
- Line count is a secondary soft warning at a generous multiple of the declared file count, never a declared budget of its own — line totals are distorted by generated files, lockfiles and formatting passes.
- v1 compares counts only. The intended path list stays prose under the required Files heading, enforced by a body-header rule; machine-comparing declared paths against the diff needs prose parsing and is deliberately out of scope.
- MindTape adopts this on its own backlog: `units = ["files"]`, a cap, and `required = "non-terminal"`. Sequencing is mandatory — ship the feature, backfill all existing tasks, only then flip `required` on, or the 51 existing artifacts all fail validation at once.
- Tests: under cap, at cap, over cap rejected at authoring; drift warning fires and stays a warning; attribution skipped with zero and with several in-flight tasks; required=non-terminal relaxes on terminal tasks.
