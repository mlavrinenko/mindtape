#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add a count aggregate to filters",
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
      enforced-practice: helps[a WIP cap becomes a rule expression a project writes],
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
  status: done(2026, 9, 28),
  links: depends-on(
    "read-estimate-units-and-kebab-case-names-in-filters.typ",
  ),
)

== Problem

+ `.mindtape/config.toml` caps work in progress with two `[[tasks.limits]]` entries.
+ 🔴 No filter can count tasks: an expression sees one task, so the cap needs its own table and checker (`crates/mindtape/src/config_limits.rs`).


== TO-BE

- 🟢 `count(<expr>)` counts loaded tasks matching `<expr>`. The argument is an expression, not a string, so no nested quoting:

```toml
[[tasks.rules]]
name     = "wip-cap"
when     = 'status == "wip"'
assert   = 'count(status == "wip") <= 1'
severity = "error"
```

- 🟢 It ranges over the whole loaded set, never a listing's narrowed rows, the same rule `ready` follows.
- 🟢 Inside `count` every field reads the task being counted, so its value never depends on the outer task: it is computed once per command, and a rule over 2000 tasks stays linear (`just bench` shows no regression).


== Changes

- `crates/mindtape/src/filter_aggregate_tests.rs` (red first): `count` over the whole set under a narrowing filter, nested inside a comparison, computed once.
- `crates/mindtape/src/filter_aggregate.rs` (new): evaluation over the loaded set and the per-command cache.
- `crates/mindtape/src/filter.rs`: the evaluation context carries the loaded set.
- `www/content/filters.typ`: document it.
- `CHANGELOG.md`: the addition.


== Alternatives

- `sum(<num-expr>, <expr>)` for budgets over several tasks in flight: dropped. This repo caps work in progress at one task, so a per-task budget already sees the whole working tree; add `sum` when a project with a wider cap needs it.
- String arguments (`count("status == \"wip\"")`): nested quoting inside TOML literals.


== Out of scope

- Retiring `[[tasks.limits]]`: `truth/tasks/retire-task-limits-in-favour-of-count-rules.typ`.


== Dogfood

This repo's two WIP limits become `count(...)` rules in the retirement task.


== Outcome

- The loaded set rides on `Readiness` (`crates/mindtape/src/ready.rs`), not a new parameter: every command already builds it once over the whole set before any filter narrows, and hands it to every set-aware evaluation. `filter.rs`'s context reaches the set through it; `filter_aggregate.rs` holds the snapshot and the per-command cache, keyed by the argument's text.
- Deviation: `count` is an error where no set exists (`Readiness::empty`: `required`, gates, the estimate `when`), not zero, so a cap never passes unmeasured. `required` rejects it at load. `[[tasks.rules]]` and `[[on.flip]]` dry-run through `expr_problem_in_set`, a one-task set of the probe; the hooks' private copy of `expr_problem` went with it.
- Deviation: `count(...)` is the left operand of a comparison only, where a field goes; `1 >= count(...)` is refused as the literal-side grammar was.
- `examples/large` asserts a `count` rule on every task, so `just bench` times it. Interleaved medians over 2000 tasks: `mt check` 98 ms before, 101 ms after, 104 ms with the rule; `mt ls` 199 ms before, 200 ms after.
- `join_kebab_names` moved to `crates/mindtape/src/expr.rs` to keep `filter.rs` under the size cap.
- The work touched 13 files against an estimate of 5: `filter_aggregate.rs` plus its tests, `filter.rs`, `ready.rs`, `expr.rs`, `lib.rs`, the two dry-run callers, the bench config, and four docs files (`filters.typ`, `skill.typ` and its generated `SKILL.md`, `CHANGELOG.md`), with `outdatty.lock`.
