#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Filter and sort parity for all standard columns",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 8,
  tags: (),
  estimate: (files: 2),
  status: done(
    2026,
    8,
    7,
  )[unified filter and sort onto the `crate::fields` registry, with `assignee` and `reviewed` sortable],
)

== Summary

Not all fields are filterable and sortable. Take difficulty: at filing, it was
neither filterable nor sortable. Fix that, and add a systematic approach so the gap cannot
return. Apply refactoring where needed.

By the time work started on this task, `difficulty` itself was already filterable
and sortable (landed in `68eedb3`, after this task arrived). The real, verified
gap was narrower. `assignee` and `reviewed` compared in a filter but silently
lacked sorting. A missing optional field (such as `kind`) sorted first ascending
while every numeric field sorted a missing value last. That was an
inconsistency, not a documented rule.

== Scope

- Added `crates/mindtape/src/fields.rs`: one shared registry (`FIELDS`,
  `get(name)`) mapping each generically comparable field (`status`, `priority`,
  `difficulty`, `kind`, `title`, `id`, `due`, `start`, `reviewed`, `file`,
  `assignee`, `depth`) to a single `fn(&Task, &Config) -> FieldValue` accessor.
- `filter::field_value` delegates to the registry instead of hand-matching field
  names, so a field wired into the registry becomes filterable by construction,
  with no second match statement to forget.
- `sort::compare_field` falls through to the same registry for every field
  without config- or readiness-aware ordering (`status`, `ready`, `priority`
  and `difficulty` stay special-cased). `sort_string_key` drops its separate
  string-key match, where an unregistered field fell back to always-equal without
  a warning.
- `SORT_FIELDS` gained `reviewed` and `assignee`. A new test,
  `sort_fields_covers_every_registry_field`, asserts that every `fields::FIELDS`
  entry appears in `SORT_FIELDS`. A future field added to the registry and
  forgotten in `SORT_FIELDS` then fails a named test instead of staying quietly
  without sort support.
- Fixed the missing-value sort inconsistency: every field sorts a missing
  value last ascending (first descending, matching the pre-existing rule for
  `priority` and `difficulty`), via a shared `cmp_field_value`.
- Updated `www/content/filters.typ` with the sortable-fields list the "Sort and
  limit" section lacked, and `skills/mindtape/SKILL.md` with `assignee` on its
  filter-fields line, a pre-existing omission.
- Out of scope: the `mt ls` display columns (`LS_COLUMNS`/`ColumnId` in
  `render/columns.rs`) are a separate, deliberately curated 7-field subset
  (documented at `output.rs:99`). Expanding which fields can render as a table or
  CSV column is a different, larger piece of work: header labels, cell glyphs and
  colour, auto-significance, width flex. It is not what "filterable and sortable"
  in the summary describes.
