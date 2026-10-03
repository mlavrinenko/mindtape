#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep a filed src-files estimate when the budget warning fires",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 9,
  ),
  difficulty: 1,
  tags: (),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[an agent over budget splits the work instead of asking to raise the estimate],
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 3),
)

== Problem

+ Run `mt wip` on a task whose diff touches more source files than `estimate.src-files`.
+ 🔴 It warns `rules.src-budget: split the remaining work into a new task and finish the part in hand`, and an agent asks whether to raise the estimate.


== TO-BE

- 🟢 The warning says the estimate stays as filed, because it records the decomposition.


== Changes

- `.mindtape/config.toml`: the `src-budget` message and comment say the estimate is never raised.


== Rollback

Revert the commit; the message loses the clause.


== Out of scope

- The rule's `assert` and its exclude list.


== Dogfood

Agents over budget here split the work and leave the estimate alone.

