#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

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
      implementable-tasks: helps[the writer gets the refusal.],
      enforced-practice: helps[a write cannot land a graph `mt check` rejects],
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
  status: done(
    2026,
    9,
    27,
  )[`check_candidates` reports a new cycle to every candidate. Pre-existing cycles never block.],
)

== AS-IS

+ In a scratch project, `mt add alpha` and `mt add beta`.
+ `mt link alpha depends-on beta` exits 0.
+ 🔴 `mt link beta depends-on alpha` exits 0 and prints `linked beta: +depends-on -> alpha.typ`.
+ `mt check` then reports `alpha: depends-on: depends-on cycle: alpha → beta → alpha`, and the tool refuses every later write to `alpha` for a violation the `alpha` write did not cause.

The pre-write seam validates the files it writes, but the cycle shows up on the other end of the edge. The refusal keeps `mt check`'s rotation (alpha → beta → alpha, reported on alpha) rather than re-rotating it onto the writer.


== Environment

- `mt` 0.3.0 and `HEAD` (`f505a12`), Linux.


== TO-BE

- 🟢 `mt link beta depends-on alpha` exits non-zero naming the cycle `beta → alpha → beta`, and both files stay byte-identical.
- 🟢 The same refusal covers `mt link --apply`, `mt import` and `mt add` with a `links` slot: any write that introduces a cycle on an `acyclic` kind.
- 🟢 A violation that already existed before the write never blocks it.


== Changes

- `crates/mindtape-cli/tests/link_e2e.rs`: the red test. Linking the reverse edge exits non-zero and leaves both files untouched.
- `crates/mindtape-cli/src/pre_write.rs`: compare project-wide link violations before and after the batch, and refuse only the new ones.
- `crates/mindtape-cli/src/link_cmd.rs`: route through that comparison if it does not already.


== Out of scope

- Changing what `mt check` reports: its cycle detection is right.
- Non-link rules that span tasks (WIP limits): a separate question.


== Dogfood

`truth/tasks/` links through `mt link` every day. A cycle there fails at the write.

