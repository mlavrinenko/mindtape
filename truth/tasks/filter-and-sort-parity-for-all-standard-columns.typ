#import "@local/mindtape:0.2.0": *

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
  )[unified filter and sort onto crate::fields registry; assignee/reviewed now sortable],
)

== Summary

Currently not all fields are filterable and sortable. For example, we can't filter or sort by difficulty.
We should fix and, but also come up with some systematic approach so it wouldn't be possible in the future.
Apply refactoring if needed.

Note: by the time this was picked up, `difficulty` itself was already filterable and sortable (landed in
`68eedb3`, after this task was filed). The real, verified gap was narrower: `assignee` and `reviewed` were
comparable in a filter but silently unsortable, and a missing optional field (e.g. `kind`) sorted first
ascending while every numeric field sorted a missing value last -- an inconsistency, not a documented rule.

== Scope

- Added `crates/mindtape/src/fields.rs`: one shared registry (`FIELDS`, `get(name)`) mapping each
  generically comparable field (`status`, `priority`, `difficulty`, `kind`, `title`, `id`, `due`, `start`,
  `reviewed`, `file`, `assignee`, `depth`) to a single `fn(&Task, &Config) -> FieldValue` accessor.
- `filter::field_value` now delegates to the registry instead of hand-matching field names, so a field
  wired into the registry is filterable by construction -- no second match statement to forget.
- `sort::compare_field` falls through to the same registry for every field without config/readiness-aware
  ordering (`status`, `ready`, `priority`, `difficulty` stay special-cased); `sort_string_key`'s separate,
  silently-lossy string-key match (an unregistered field fell back to always-equal) is gone.
- `SORT_FIELDS` gained `reviewed` and `assignee`. A new test, `sort_fields_covers_every_registry_field`,
  asserts every `fields::FIELDS` entry is listed in `SORT_FIELDS` -- so a future field added to the
  registry and forgotten in `SORT_FIELDS` fails a named test instead of staying quietly unsortable.
- Fixed the missing-value sort inconsistency: every field now sorts a missing value last ascending (first
  descending, matching `priority`/`difficulty`'s pre-existing rule), via a shared `cmp_field_value`.
- Updated `www/content/filters.typ` (added the sortable-fields list the "Sort and limit" section lacked)
  and `skills/mindtape/SKILL.md` (added `assignee` to its filter-fields line, a pre-existing omission).
- Out of scope: `mt ls`'s *display* columns (`LS_COLUMNS`/`ColumnId` in `render/columns.rs`) are a
  separate, deliberately curated 7-field subset (documented at `output.rs:99`) -- expanding which fields
  can render as a table/CSV column is a different, larger piece of work (header labels, cell glyphs/color,
  auto-significance, width flex) and not what "filterable and sortable" in the summary describes.
