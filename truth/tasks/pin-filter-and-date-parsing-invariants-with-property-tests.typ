#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Pin filter and date parsing invariants with property tests",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling", "tests"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[parse/eval invariants are stated over generated input],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 1),
)

== Problem

+ Read the filter language tests under `crates/mindtape/src/filter*` and the date/recur tests.
+ 🔴 Those parsers take open input (expressions, dates, rrule strings) but every test samples a handful of literals by hand; no test states the invariant over all inputs, so wrong specs and parse/eval asymmetries survive both the suite and the sweep.
+ Property-based testing finds the spec you forgot to write, where mutation testing finds the assertion you forgot; the discussion's example was filters, recur and date parsing.

== TO-BE

- 🟢 `proptest` is a dev-dependency with a small strategy module for generated filter expressions, dates and rrule strings.
- 🟢 Invariants are pinned over generated inputs (parse then eval round-trips, accepted rrules round-trip through the RRULE crate, malformed input errors rather than panics), with shrinking naming a minimal counterexample.
- 🟢 Failures are reproducible from the persisted proptest seed file.

== Changes

- `Cargo.toml` `[workspace.dependencies]` and `crates/mindtape/Cargo.toml` `[dev-dependencies]`: add `proptest`.
- `crates/mindtape/src/filter_tests.rs` (or a new `filter_proptests.rs`): expression generator plus the parse/eval invariants.
- `crates/mindtape/src/recur*`'s tests: date/rrule invariants.
- Keep the proptest regression seed file committed.

== Rollback

Remove the dependency and the property modules; the example tests are untouched.

== Out of scope

- Replacing example-based tests: both stay; properties state invariants, examples document behaviour.
- `cargo-fuzz`/`afl.rs` and coverage-guided fuzzing: separate decision, not this task.

== Dogfood

A filter expression the hand-written tests accepted but eval mishandles shrinks to a minimal case instead of reaching a user bug.
