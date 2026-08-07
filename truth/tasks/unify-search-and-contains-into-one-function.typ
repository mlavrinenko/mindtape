#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "unify search and contains into one function",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli", "docs"),
  estimate: (files: 6),
  status: done(2026, 8, 7)[landed 2026-08-07; just check green on merged HEAD],
)

== Summary

The filter language carries two overlapping text predicates.
`contains(field, "substr")` matches one field; `search("query")` is documented
in `www/content/filters.typ` as "Full-text search across the task" but matches
`title` and `id` only. `Task.body` holds the whole prose body verbatim, is
loaded already, and ships in `--json` — it is simply never consulted. Measured
on this backlog: `mt ls 'search("tombstone")'` returns 1 task where
`rg -l tombstone truth/tasks/` finds 6 files.

Collapse both into one variadic function, needle first:

```
contains("bug")                 // default haystack
contains("bug", title)          // one field
contains("bug", title, body)    // several
```

One function, one argument order, one field vocabulary. `search()` goes away.

== Scope

- Replace the `"search"` and `"contains"` arms in `crate::filter` with a single
  `contains` arm: first argument a string literal (the needle), remaining
  arguments bare field identifiers, variadic and optional. Case-insensitive
  substring throughout, as `contains` is today.
- Default haystack when no field is named: `title`, `id`, `body`. That is the
  prose surface — tags have `has_tag()`, and every scalar field has a
  comparison operator, so neither belongs in an implicit text sweep.
- Declare the searchable-field vocabulary as one constant, the way
  `crate::fields::FIELDS` is the single registry behind filtering and sorting.
  It is `FIELDS` rendered as strings, plus `body` — which is text-only, never
  comparable or sortable, so it extends the vocabulary without entering
  `FIELDS`. Do not hand-match field names in the `contains` arm; that drift is
  exactly what `fields.rs` was created to end.
- The body haystack is `Task.body`, which is already marker-free: a hit on a
  `tags:` value or a link target in the `#show: task.with(…)` head is not a
  prose hit and must not count.
- Reject the old spellings with an error that teaches the new one, rather than
  silently accepting either:
  - `contains(title, "bug")` (identifier first) names the needle-first order.
  - `search("bug")` names `contains`.
  Pre-release, so breaking the old order is fine; leaving two accepted orders
  is not.
- Shape the signature so a future regex sibling is a drop-in:
  `matches(pattern, ...fields)` with identical argument structure. Not in this
  task — an infix `title regexp "…"` is not expressible (evalexpr parses it as
  a call to `title`), and substring plus `rg` covers the need today. Recorded
  here so the shape is chosen deliberately rather than rediscovered.
- Tests: a body-only match, a marker-head non-match, an explicit single field,
  several fields, the default haystack, an unknown field name, and both
  rejected legacy spellings. `filter_tests.rs` sits at 497 of its 500-line cap,
  so these land in a new `#[path]` sibling rather than growing it.
- Docs: `www/content/filters.typ` (both rows collapse to one) and
  `skills/mindtape/SKILL.md` (its filter-function line lists both today).
