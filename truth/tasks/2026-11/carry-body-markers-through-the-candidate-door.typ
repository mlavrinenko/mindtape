#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Carry body markers through the candidate door",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("mt", "tests", "no-ui-ux", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: helps[a write path and a load path expose the same marker list.],
      enforced-practice: helps[mt add sees the markers the goal check reads.],
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: helps[the candidate door and the loader share one marker list instead of two evals.],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
  links: parent("rewire-goals-through-typst-markers.typ")
    + depends-on("expose-task-body-markers-to-check-functions.typ"),
)

== AS-IS

+ 🔴 `mt add` builds a candidate through `validate/candidate.rs`. `eval_marker` keeps only the first `mindtape.task` value and drops every other marker.
+ File a task with body markers. 🔴 `check_candidates` runs the goal check with an empty marker list, so a scripted `mt add` refuses a valid task.
+ Run `mt import`, `mt link`, `mt recur` or `mt set`. 🔴 They reach the same door, so the same false refusal returns.

== TO-BE

- 🟢 `eval_marker` returns the marker list alongside the task marker, and the candidate assembly uses the same `assemble_task` input a load uses.
- 🟢 This leaf leaves the commit state unchanged: `assemble_task` returns a `Task` carrying the markers, and its cache lives in memory per run.
- 🟢 `mt add`, `mt import`, `mt link`, `mt recur` and `mt set` see the same body markers `mt check` sees.
- 🟢 A test files a marker-bearing task through `mt add` and runs the marker check on it. Both paths see the same markers.
- 🟢 No goal vocabulary enters the candidate layer.

== Changes

- `crates/mindtape/src/validate/candidate.rs`: `eval_marker` returns the full marker list. The candidate keeps it for `check_all`.
- `crates/mindtape/src/validate/candidate_tests.rs`, `crates/mindtape-cli/tests/it/add*`, `crates/mindtape-cli/tests/it/commit*`: a marker-bearing add, a scripted add and a range check.
- `CHANGELOG.md`: an entry under `[Unreleased]`.

== Out of scope

- The loader, cache and eval projection. The exposure leaf owns them.
- The goal DSL and the live rule switch.
- A second evaluation of the task in the candidate path.

== Dogfood

A scripted `mt add` files a task with goals, and the same task passes `mt check` immediately after.
