#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Configurable `ls` column order",
  status: done(2026, 6, 27),
  priority: ice(impact: 5, confidence: 0.8, ease: 6),
  tags: ("cli",),
  depends-on: (link("ls-hide-empty-columns.typ"),),
)

== Summary

`mindtape ls` renders columns in a fixed order (`#`, STATUS, READY, PRIORITY,
DUE, TITLE). Let the user choose the order — and, by omission, the selection —
through config.

== Scope

- Add `columns_order: Vec<String>` to `LsConfig`
  (`crates/mindtape/src/config.rs`). TOML form under `[ls]`:
  `columns-order = ["#", "status", "priority", "due", "title"]`.
- Empty (unset) means the shipped built-in order. When set, it both selects and
  orders: only the listed columns are candidates for rendering, in the given
  sequence.
- Validate in `Config::validate` against the canonical column-name set defined
  by the `ls-hide-empty-columns` task: an unknown name or a duplicate errors.
- Drive `crates/mindtape-cli/src/render/table.rs` (already column-list-driven
  after the sibling task) off the resolved order. `title` keeps its flex-width
  role wherever it sits; non-last columns pad normally and the per-line
  `trim_end` handles the trailing column.

== Notes

- Precedence with per-column visibility: `columns-order` decides set + order;
  per-column `visibility` (`auto` | `visible` | `hidden`) then decides whether
  each listed column actually renders. A column absent from `columns-order`
  never renders even if its `visibility = "visible"`. Document this in
  `default-config.toml`.
- Depends on `ls-hide-empty-columns` for the column descriptor refactor and the
  shared, validated column-name vocabulary; land that first.
- Tests (`config_tests.rs` / `table.rs`): a reordered list renders in that
  order; omitting a column excludes it; unknown / duplicate names fail
  validation.
