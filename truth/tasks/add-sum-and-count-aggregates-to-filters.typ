#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add sum and count aggregates to filters",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[WIP caps and summed budgets become rule expressions a project writes],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[set-level checks stop needing their own config tables],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
  links: depends-on(
    "compare-estimate-units-and-compute-arithmetic-in-filters.typ",
  ),
)

== Problem

+ Two tasks are `wip`, each with `estimate: (files: 3)`, and the working tree touches 5 files.
+ 🔴 No filter can say "the sum of the in-flight budgets": an expression sees one task, so summing lives in `crates/mindtape/src/estimate_drift.rs`, and counting lives in `[[tasks.limits]]` (`crates/mindtape/src/config_limits.rs`).


== TO-BE

- 🟢 `count(<expr>)` counts loaded tasks matching `<expr>`; `sum(<num-expr>, <expr>)` sums a numeric expression over them. The arguments are expressions, not strings, so no nested quoting:

```toml
[[tasks.rules]]
name     = "wip-cap"
when     = 'status == "wip"'
assert   = 'count(status == "wip") <= 1'
severity = "error"
```

- 🟢 An aggregate ranges over the whole loaded set, never a listing's narrowed rows, the same rule `ready` follows.
- 🟢 Inside an aggregate every field reads the task being counted, so its value never depends on the outer task: each aggregate is computed once per command, and a rule over 2000 tasks stays linear (`just bench` shows no regression).


== Changes

- `crates/mindtape/src/filter_aggregate_tests.rs` (red first): `count`, `sum` with absent values skipped, whole-set range under a narrowing filter, memoization.
- `crates/mindtape/src/filter_aggregate.rs` (new): evaluation over the loaded set and the per-command cache.
- `crates/mindtape/src/filter.rs`: the evaluation context carries the loaded set.
- `www/content/filters.typ`: document both.
- `CHANGELOG.md`: the additions.


== Alternatives

- String arguments (`count("status == \"wip\"")`): nested quoting inside TOML literals.
- Keep `[[tasks.limits]]` as the only set-level check: one aggregate, one table, and every new aggregate a new table.
- `min`/`max`/`avg` now: add when a rule needs one.


== Out of scope

- Retiring `[[tasks.limits]]`: `truth/tasks/retire-task-limits-in-favour-of-count-rules.typ`.


== Dogfood

This repo's two WIP limits become `count(...)` rules in the retirement task.

