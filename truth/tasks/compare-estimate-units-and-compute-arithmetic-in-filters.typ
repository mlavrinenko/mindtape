#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Compare estimate units and compute arithmetic in filters",
  kind: "feat",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a budget or bound on any unit is one rule expression, not a Rust checker],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the numeric vocabulary the rule retirements need lives in the language once],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 9, 27),
)

== Problem

+ `mt ls 'estimate.files > 3'`.
+ 🔴 It is refused: `estimate` supports only `has(estimate)`/`miss(estimate)` (`crates/mindtape/src/filter_predicates.rs`), so no rule can compare an estimate, and bounds on it needed the bespoke `[[tasks.estimate.max]]`/`min` checker.
+ `mt ls 'priority * 2 > 10'`.
+ 🔴 The filter walker has no arithmetic; only `crates/mindtape/src/framework_eval.rs` evaluates `+ - * /`.


== TO-BE

- 🟢 `estimate.<unit>` reads a declared unit as a number for any unit name; mindtape knows none of them. An absent unit behaves like any absent numeric field, and `has(estimate.files)` tests presence.
- 🟢 `+ - * /` work over numbers anywhere in a filter, rule or view: `lines("worktree") <= 200 * estimate.files`. Division by zero and a non-numeric operand are errors naming the expression.
- 🟢 A unit outside `[tasks.estimate] units` in a saved expression is a config error at load, like an unknown field.


== Changes

- `crates/mindtape/src/filter_predicates_tests.rs` (red first): `estimate.files > 3`, `has(estimate.files)`, arithmetic precedence, division by zero.
- `crates/mindtape/src/filter_predicates.rs`: the dotted `estimate.<unit>` lookup.
- `crates/mindtape/src/filter.rs`: arithmetic nodes in the walker, sharing operand coercion with comparisons.
- `www/content/filters.typ`: document both.
- `CHANGELOG.md`: the additions.


== Alternatives

- A per-unit field list in config (`numeric-fields = ["files"]`): a second vocabulary to keep in step with `units`.
- Reuse `framework_eval.rs` wholesale: it evaluates intervals over framework inputs, a different domain; share only what is literally the same.


== Out of scope

- Aggregates over tasks: `truth/tasks/add-sum-and-count-aggregates-to-filters.typ`.
- Retiring `[[tasks.estimate.max]]`/`min`: `truth/tasks/retire-the-drift-check-and-estimate-bounds-in-favour-of-rules.typ`.


== Dogfood

This repo's estimate bounds become `assert = 'estimate.files <= 5'` rules once the retirement task lands.

