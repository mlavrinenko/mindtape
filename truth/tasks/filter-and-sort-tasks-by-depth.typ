#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Filter and sort tasks by depth",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli", "tests"),
  estimate: (files: 7),
  status: proposed(2026, 8, 3),
)

== Summary

Allow listing tasks by their nesting depth under a root: root tasks (no sub-folder) match depth = 0, ones in one sub-folder match depth = 1, and so on. Filed from consumer feedback: they want to list all root tasks saved directly in the tasks folder (with no sub-folders).

== Scope

depth on the filter field set and on the sort fields, plus regression tests

== Orientation

Task stores `file` as a project-root-relative `PathBuf` (e.g. `truth/tasks/sub/x.typ`), so depth is the number of folder segments between the task's root and the file — the file sitting directly under the root is depth 0. The loader already computes this kind of location data: `crates/mindtape/src/derive.rs` `folder_segments(rel_within_root)` turns the root-relative path into its folder segments, and `crates/mindtape/src/loader.rs` `collect_items` derives `rel_within` per file at load time. Model precedent: `derived_tags` is injected onto `Task` in `loader.rs:assemble_task`; a `depth: u8` slot would follow the same path (set once at load, alongside `folder_tags`), so it is consistent and cheap even with several roots.

Affected files:

- `crates/mindtape/src/filter.rs` — `field_value` (`field_value` match on line 364): add `"depth" => FieldValue::Num(...)` so `mt ls 'depth == 0'` and the numeric comparisons described in the case work; `filename` bookkeeping is via the `comparison`/`nullary` walk, unchanged.
- `crates/mindtape/src/sort.rs` — `SORT_FIELDS` const (line 18) and `sort_string_key`/`compare_field` (line 85): add `"depth"` so `--sort depth[:asc|:desc]` works (a numeric field, so it belongs in `compare_field` beside priority/difficulty rather than `sort_string_key`).
- `crates/mindtape/src/model.rs` — add a `depth` field on `Task` (or a getter), following `derived_tags`'s precedent; `from_marker` starts it empty/zero.
- `crates/mindtape/src/loader.rs` — `collect_items` (line 171) already walks every `.typ` file and computes `rel_within`; compute depth there from the same path and attach it like `derived_tags`.

Consult `filter-and-sort-tasks-by-file.typ` in this root: it is the closest prior art (adding a location-derived field to the filter language and `SORT_FIELDS`, both arms reading one shared source). Regression tests live in `filter_tests.rs`, `filter_dispatch_tests.rs`, and `sort_tests.rs`; `filter-dispatch` covers `mt ls '<expr>'` end-to-end.

== Decision: `==`, never `=`

The supported query is `mt ls 'depth == 0'` (and the other binary comparisons `!=`/`<`/`>`/`<=`/`>=`). A single `=` is not supported and will not be: `evalexpr` reads `=` as an assignment operator, and supporting it would mean fighting the eval engine. A bare `mt ls 'depth = 0'` stays rejected as `unsupported in filter: Assign`. The task's motivating example is written with `==`.
