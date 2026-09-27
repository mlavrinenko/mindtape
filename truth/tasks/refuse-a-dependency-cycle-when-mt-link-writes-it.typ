#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Refuse a dependency cycle when mt link writes it",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "cli"),
  estimate: (files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[agents linking tasks get the refusal at the write, not a poisoned neighbour later],
      enforced-practice: helps[a write can no longer land a graph `mt check` rejects],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the pre-write seam checks what the write breaks, not only the file it writes],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
)

== Problem

+ In a scratch project, `mt add alpha` and `mt add beta`.
+ `mt link alpha depends-on beta` exits 0.
+ 🔴 `mt link beta depends-on alpha` exits 0 and prints `linked beta: +depends-on -> alpha.typ`.
+ `mt check` then reports `alpha: depends-on: depends-on cycle: alpha → beta → alpha`, and every later write to `alpha` is refused for a violation the `alpha` write did not cause.

The pre-write seam validates the files being written, but a cycle is reported on the other end of the edge.


== Environment

- `mt` 0.3.0 and `HEAD` (`f505a12`), Linux.


== TO-BE

- 🟢 `mt link beta depends-on alpha` exits non-zero naming the cycle `beta → alpha → beta`, and both files stay byte-identical.
- 🟢 The same refusal covers `mt link --apply`, `mt import` and `mt add` with a `links` slot: any write that introduces a cycle on an `acyclic` kind.
- 🟢 A violation that already existed before the write never blocks it.


== Changes

- `crates/mindtape-cli/tests/link_e2e.rs`: the red test — linking the reverse edge exits non-zero and leaves both files untouched.
- `crates/mindtape-cli/src/pre_write.rs`: compare project-wide link violations before and after the batch, and refuse only the new ones.
- `crates/mindtape-cli/src/link_cmd.rs`: route through that comparison if it does not already.


== Out of scope

- Changing what `mt check` reports: its cycle detection is right.
- Non-link rules that span tasks (WIP limits): a separate question.


== Dogfood

`truth/tasks/` links through `mt link` every day; a cycle there now fails at the write.

