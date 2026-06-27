#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Hide empty `ls` columns, per-column configurable",
  status: wip(2026, 6, 27),
  priority: ice(impact: 6, confidence: 0.85, ease: 5),
  tags: ("cli",),
)

== Summary

`mindtape ls` always renders STATUS, PRIORITY and DUE even when no task in the
list carries that field, wasting horizontal space on a column of blanks. Only
READY auto-hides today (bespoke `show_ready` gate). Generalize: a column whose
cells are all insignificant is hidden by default, and the user can pin any
column visible or force it hidden through per-column config.

== Scope

- Add a per-column settings table to `LsConfig`
  (`crates/mindtape/src/config.rs`):
  `columns: BTreeMap<String, ColumnConfig>` where
  `ColumnConfig { visibility: Visibility }` and
  `Visibility = Auto | Visible | Hidden` (default `Auto`). The struct is a map
  value, not a scalar, so future column knobs (e.g. `width`) extend it without a
  breaking shape change. TOML form:
  `[ls.columns.priority]` / `visibility = "auto"` (`auto` | `visible` |
  `hidden`).
- Define the canonical column-name set in the `mindtape` lib (one source of
  truth, consumer-blind): `#`, `status`, `ready`, `priority`, `due`, `title`.
  Validate `[ls.columns]` keys against it in `Config::validate` — an unknown
  column name errors with the legal list (matches the `deny-unknown-fields`
  policy).
- Refactor `crates/mindtape-cli/src/render/table.rs` from the hardcoded `Row` /
  `Widths` structs to a column-list-driven renderer. Extract a column descriptor
  (suggest `render/columns.rs` to stay under the 500-line cap): each column
  knows its key, header label, how to build a `CellSpec` from
  `(handle, task, config, readiness)`, and its auto-significance test.
- Visibility resolution per column: `Visible` always renders, `Hidden` never,
  `Auto` renders only when the column is significant. Significance:
  - `#` and `title` always significant (never blank).
  - `ready` significant when some task is blocked (preserve today's uncluttered
    behaviour — a backlog of all-ready tasks keeps READY hidden).
  - `status`, `priority`, `due` significant when any cell has non-empty text.
- Width + layout stays as today otherwise: natural width per column, `title` is
  the flex column absorbing remaining terminal width. Pad every cell to its
  width then `trim_end` each emitted line so the trailing column carries no run
  (replaces the title-is-last special case, since title need not be last once
  `ls-columns-order` lands).

== Notes

- Default behaviour flips to hide-empty: with no `[ls.columns]` config, STATUS /
  PRIORITY / DUE drop when the whole list lacks them. This is the requested
  default; call it out in `default-config.toml` and `README`.
- Keep the column-name constant and `Visibility` enum in the lib so the sibling
  `ls-columns-order` task and any consumer share one validated vocabulary.
- Tests (`table.rs`): all-empty PRIORITY/DUE hidden under `Auto`; `Visible`
  forces a blank column to render; `Hidden` drops a populated column; READY
  still appears only when blocked; config validation rejects an unknown column
  key.
