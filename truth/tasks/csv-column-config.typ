#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "CSV output honours `[ls.columns]` / `columns-order` like the table renderer",
  status: wip(2026, 7, 4),
  priority: ice(impact: 3, confidence: 0.7, ease: 6),
  tags: ("cli",),
)

== Summary

CSV output hardcodes a fixed 5-column set `["id", "status", "priority", "due",
"title"]` (`crates/mindtape-cli/src/output.rs:54-68`), ignoring the
`[ls.columns]` / `columns-order` config the table renderer honours. It silently
drops `kind`, `estimate`, `tags`, and every `x-<ns>` extra (JSON exposes them;
CSV does not).

== Scope

- Drive the CSV column set + order from the same resolved column configuration the
  table path uses (`config_ls.rs` `LS_COLUMNS` / `columns-order` /
  `Visibility`), so CSV and table agree on which columns render.
- Decide and document the mapping for fields that have no fixed table column
  (`tags`, `estimate`, `x-*`): either extend the shared column model to name them
  or keep CSV to the configured column set and note the limitation. Prefer
  reusing the shared model over a CSV-only list.
- Keep JSON untouched (it already emits every field).

== Tests

- `mt ls --format csv` with a custom `columns-order` emits exactly those columns
  in that order.
- Default config → CSV matches the default column set (no silent-drop
  regressions for the documented columns).

== Notes

- Independent of the config-layering tasks; touches only the CLI output path.
- Coordinate file boundaries with `ready-overdue-display-config.typ` if run in
  parallel — both live under `render`/`output`; keep the edits disjoint.
