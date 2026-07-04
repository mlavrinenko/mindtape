#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Config seam for the READY column and overdue accent (glyph / colour / label)",
  status: proposed(2026, 7, 4),
  priority: ice(impact: 4, confidence: 0.7, ease: 5),
  tags: ("cli",),
  depends-on: (link("config-embedded-defaults.typ"),),
)

== Summary

Every rendered column resolves glyph / colour / label through config (`Legend` /
`NumericField`) except the built-in READY column, which hardcodes `▶` / `🔒`,
green / red, "ready" / "blocked" (`crates/mindtape-cli/src/render/columns.rs:141`),
and the overdue due-date accent hardcodes `red` (`columns.rs:178`). Give them the
same config seam the other columns have, with the current values as the shipped
floor default.

== Scope

- Add a small `[ls.ready]` (or `[ls.columns.ready]`) config block: `ready-glyph`,
  `ready-color`, `ready-label`, `blocked-glyph`, `blocked-color`, `blocked-label`,
  library-side in `config_ls.rs`. Add `[ls] overdue-color`.
- Default the new keys FROM the embedded floor (foundation task), so behaviour is
  unchanged out of the box and the defaults are visible in `mt config show`.
- `ready_cell` (`columns.rs:141`) and `due_cell` (`columns.rs:178`) read the
  configured values instead of literals.

== Tests

- Unset config → the current glyphs/colours/labels (floor supplies them).
- Overridden `[ls.ready] ready-glyph = ">"` etc. renders the override.
- Overdue accent honours `[ls] overdue-color`.

== Notes

- Lower priority than the config-layering core; a display polish that the layering
  makes cheap (new keys get floor defaults for free).
- While here, note but do NOT expand scope to: the two inconsistent Emoji-degrade
  rules (`render/glyph.rs` vs `render/table.rs`) and CSV column config — those are
  separate tasks.
