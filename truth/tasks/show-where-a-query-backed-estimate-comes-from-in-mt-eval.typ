#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show where a query-backed estimate comes from in mt eval",
  kind: "feat",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: helps[a reviewer sees which part of an estimate is a guess and which part the code says],
      implementable-tasks: neutral,
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
  status: done(2026, 10, 25),
  links: depends-on(
    "resolve-an-estimate-from-a-manual-count-plus-file-queries.typ",
  ),
)

== Problem

+ Once query-backed estimates resolve, file a task with `estimate: (src-files: count(2, grep(...)))` and run `mt eval <stem>`.
+ 🔴 Only the resolved total shows, so a reviewer can't tell the manual part from what each query matched, or check a query that matches nothing.

== TO-BE

- 🟢 `mt eval` prints each query-backed unit as its manual part, each query with its match count, and the total.
- 🟢 `mt eval --json` carries the same breakdown beside the resolved number.

== Changes

- `crates/mindtape-cli/src/cmd/eval_tests.rs`: the red test first, a composite estimate printing its breakdown.
- `crates/mindtape-cli/src/cmd/eval.rs`: render the breakdown.
- `crates/mindtape/src/estimate_queries.rs`: expose the per-query counts it already computes.

== Alternatives

- A new `mt estimate <stem>` command. One more verb for what `eval` already shows about a task.

== Out of scope

- Listing the matched files themselves; the query re-runs anywhere.

== Dogfood

This repo reviews a judgment task's fan-out with `mt eval` before flipping it to wip.
