#import "/truth/prelude/2/lib.typ": *

#show: task.with(
  title: "Stop the filter allocating a field name four times per comparison",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  links: (
    (
      task: "mint-the-filter-match-instruction-ceiling-from-its-measurement.typ",
      kind: "surfaced-by",
      note: none,
    ),
  ),
  tags: ("found", "tooling", "stateless"),
  assignee: "agent",
  status: done(2026, 11, 27),
)

== AS-IS

A finding records one fact about one path, with the evidence that shows it.

+ Bisecting `filter/match/2000` against `0fb64102` while re-minting its instruction ceiling
+ 🔴 #raw("crates/mindtape/src/filter/compare.rs"): `comparison()` restores the left side's field name four times per task, and formats an error for the right side that it discards

`priority >= 3` reads `priority` four times: in `refuse_derived_collection`,
in the `role` test, in `check_status`, and in `operand`. Each read allocates a
`String`, because `restore_name` replaces unconditionally:

```rust
pub(crate) fn restore_name(name: &str) -> String {
    name.replace(NAME_HYPHEN, "-")
}
```

The right side is worse. `3` is not an identifier, so `extract_identifier`
builds `format!("expected a field, got {}", describe(node))` and
`refuse_derived_collection` drops it with `let Ok(name) = … else { return Ok(()) }`.

```
$ callgrind_annotate --inclusive=yes --auto=no --threshold=99 \
    target/iai/mindtape/instructions/cpu_bound/filter_match.n2000/callgrind.filter_match.n2000.1999364.t01.p1.out
 4,120,000 (11.50%)  crates/mindtape/src/filter/expr.rs:mindtape::filter::expr::extract_identifier
 2,916,000 ( 8.14%)  crates/mindtape/src/filter/predicates.rs:mindtape::filter::predicates::refuse_derived_collection
 1,636,000 ( 4.57%)  ???:alloc::fmt::format::format_inner
 1,142,000 ( 3.19%)  crates/mindtape/src/filter/expr.rs:mindtape::filter::expr::describe
   942,000 ( 2.62%)  crates/mindtape/src/filter/compare.rs:mindtape::filter::compare::check_status
```

== TO-BE

- 🟢 `restore_name` borrows a name that doesn't hold a `NAME_HYPHEN`, so `priority >= 3` doesn't allocate per task.
- 🟢 `refuse_derived_collection` reads the name without formatting an error, so the literal side of a comparison costs a match and nothing else.
- 🟢 `filter/match/2000` retires about 3 M fewer instructions, under 34 M. The ceiling comes back down to the one `0fb64102` minted.
- 🟢 Every refusal still reports the field it found, unchanged.

```
$ just bench::instructions
Iai-Callgrind result: Ok. 3 without regressions; 0 regressed; 3 benchmarks finished
```

== Changes

- `crates/mindtape/src/filter/expr.rs`: `restore_name` returns `Cow<str>`, borrowing when the name doesn't hold a `NAME_HYPHEN`. Add `identifier_of`, which returns `Option<Cow<str>>` and never formats.
- `crates/mindtape/src/filter/predicates.rs`: `refuse_derived_collection` reads through `identifier_of`, so the literal side of a comparison formats nothing.
- `crates/mindtape/src/filter/compare.rs`: `comparison_field` and the `role` test read through `identifier_of`.
- `crates/mindtape/src/filter/mod.rs`, `crates/mindtape/src/commit/expr.rs` and `crates/mindtape/src/at/board.rs`: adapt the callers that want a `String`.
- `crates/mindtape/benches/instructions.rs`: lower `FILTER_MATCH_CEILING` to the new measurement plus 5%.

== Out of scope

- The third of the case that is dropping the loaded set. That is the bench's shape, not the filter's.
- `framework::eval::context_of`, which builds a `HashMapContext` per task. The sort pays it too, and it is a change of its own.
- `sort/default/2000`. It resolves one key per task and doesn't read an identifier per comparison.

== Dogfood

`just bench::instructions` is the gate this repo runs before a filter change,
and every `mt ls` pays this path once per task.

== Goals

+ #Helps(maintainer, to: keep-changes-safe)[because a gate that sits 11% above its case stops catching the regressions it exists for].
+ #Neutral(read-tasks-at-a-glance, guide-the-implementer, run-checks-at-the-write, keep-the-engine-agnostic, adopt-in-one-line, beat-markdown, keep-plain-files, explain-itself, match-docs-to-code).
