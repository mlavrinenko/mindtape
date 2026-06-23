#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add `now()` to the filter language",
  status: proposed(2026, 6, 23),
  priority: ice(impact: 6, confidence: 0.95, ease: 9),
  tags: ("tooling",),
)

== Summary

The filter language has no way to compare a date field against today.
`now()` should return today's ISO date string (`YYYY-MM-DD`), enabling
comparisons like `start <= now()` or `due < now()`.

== Scope

- Extend `comparison()` in `crates/mindtape/src/filter.rs` to accept
  `now()` on the right-hand side of a comparison via a new `eval_rhs()`
  helper (handles both literals and the `now()` call).
- `now()` delegates to `crate::date::today_iso()`.
- Update the module doc comment to advertise `now()`.
- Add regression tests in `filter_tests.rs` covering:
  - `start <= now()` matches when start is yesterday, fails when tomorrow.
  - `due < now()` on a task with a past due date.

== Notes

- `now()` is only valid on the right-hand side of a comparison.
  Using it as a bare identifier or inside another function (e.g. `stale(now())`)
  should return a clear `FilterError::Invalid`.
- No new crates; `crate::date::today_iso()` already exists.
