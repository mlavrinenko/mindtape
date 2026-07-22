#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "open set on flip",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 7,
  ),
  tags: (),
  status: proposed(2026, 7, 22),
)

== Summary

Print the current open task set when a task is flipped into a non-terminal status, so an agent starting work sees what else is in flight without a separate command and without a materialised index. This replaces the rejected idea of every task linking every other open task: that is O(n squared) edges, it corrupts the link graph that readiness and cycle detection read, it needs every existing file rewritten on each add, and the enumerate-everything-adjacent signal is the one that correlated negatively with merge success. The need is context delivery at the right moment, not data modelling — and flip is the moment, because once done is unreachable from wip the state machine forces the agent through it.

== Scope

- After a successful `mt flip` into a non-terminal status, print the tasks matching a configured view.
- `[flip] context-view` names it, defaulting to the `default` view. No status value is hardcoded in Rust.
- Suppress under `--quiet` and when output is not a terminal, so scripted flips stay clean.
- Nothing is written to disk. The historical open set is already recoverable — tasks are files in git, so `mt ls` at the flipping commit reconstructs it exactly.
- Tests: prints on a flip into a non-terminal status, silent into a terminal one, honours a custom context-view, silent when piped.
