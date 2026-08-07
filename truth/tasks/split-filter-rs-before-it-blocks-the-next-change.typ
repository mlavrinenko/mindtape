#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "split filter rs before it blocks the next change",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "tooling"),
  estimate: (files: 4),
  status: done(2026, 8, 8)[landed 2026-08-08; just check green on merged HEAD],
  links: related("unify-search-and-contains-into-one-function.typ")[
    the change that consumed the remaining budget
  ],
)

== Summary

`crates/mindtape/src/filter.rs` sits at 497 of its 500-line cap and
`filter_tests.rs` at 495. The `contains()` unification landed only after
trimming doc comments to squeeze back under — the change was paid for in prose,
not in structure, which is the wrong currency. The next edit to either file
starts by hitting the wall.

This is the failure the file-size gate exists to prevent, arriving one change
late: a cap reached by attrition rather than by design means the next author
spends their change on rewording instead of on the work.

== Scope

- Split `filter.rs` along a real seam, not at the line count. Candidates, in
  rough order of independence: the function-call dispatch arms (`has`, `miss`,
  `has_tag`, `contains`, `stale`, `links`, `in_view`), the comparison/field
  resolution path, and the `Readiness` dependency predicates.
- `filter_links.rs`, `filter_views.rs` and `fields.rs` already show the
  established seam shape here — follow it rather than inventing a new one.
- Split the test siblings to match, so each module's tests sit beside it.
  `just fix-check` ejects oversized inline tests via `ejectest`.
- Land the split on its own, with no behaviour change in the same commit, so
  the diff is reviewable as a move.
- Afterwards run `linecop --baseline 85` and confirm nothing in the filter
  family is near its cap.

== Notes

`config.rs` (499) and `rm.rs` (494) are in the same position and worth the same
treatment, but each is its own task — do not bundle them.
