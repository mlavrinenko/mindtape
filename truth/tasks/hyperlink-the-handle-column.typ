#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Hyperlink the handle column",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 12),
  links: depends-on("hyperlink-the-file-column-with-osc-8.typ"),
  status: done(2026, 7, 28),
)

== Summary

Once OSC 8 wrapping exists, the `#` handle is the natural thing to click. It is
the leftmost cell, it is already what you read to identify a row, and unlike
the `file` column it costs no horizontal space — the link rides on characters
that are on screen regardless. A listing could then be click-navigable without
showing a path column at all.

== Decisions

- Reuses the `[ls] hyperlinks` switch rather than introducing a second opt-in
  surface. One question ("do links work in your terminal") should have one
  answer.
- The target is the task's own file, built through the same
  `[ls] hyperlink-target` template.
- Both `#` and `file` link. The two open questions this task was filed with —
  whether per-column control is wanted, and whether linking `#` makes the
  `file` column redundant — are settled: no per-column `link` key, and `file`
  stays exactly as it is. Linking `#` is additive. Someone who wants the path
  on screen still opts the column in; someone who does not gets a
  click-navigable listing with no extra width spent.
- No new configuration. `[ls] hyperlinks` remains the single switch, and the
  set of linked columns is fixed rather than declared.

== Scope

- `build_columns` in `table.rs` currently attaches `link_uris` when
  `id == ColumnId::File`. Extend that to `Handle` as well; the URI is per row,
  not per column, so both columns share one target list and it is built once.
- The interactive path is the risk. `apply_highlight` rewrites the handle
  column's cells for `mt ls -i`, and the same characters are what a user types
  to select a row. Highlighting and linking must compose without the escape
  payload leaking into the width accounting or into what the key loop reads
  back.
- Tests: `#` wrapped when hyperlinks are on and not otherwise; both columns
  carrying the same target for a given row; layout byte-identical once the
  sequences are stripped, matching the existing width-accounting test; the
  interactive highlight still correct with links active.

== Outcome

The feared conflict did not exist. The key loop reads real key events from
crossterm, never the rendered screen, so the escape payload on a handle cell
cannot reach what the loop matches against — and the wrap already sat after
width fitting in `render_grid`, which is where highlight applies too. `#` and
`file` compose by construction rather than by care.

`is_linked_column` states the fixed set in one place, and `build_columns`
builds the per-row target list once — only when a linked column is actually
visible — and clones it into each. Two calls could have disagreed if
`[ls] hyperlink-target` or the handles drifted between them.

Estimate 6 declared, 12 touched, and unlike the previous overrun this one is
not a doc reconcile: half the count is test files. Filed at 6 by reasoning
from the production diff, which really is about two files. That is the wrong
way to size a task in a repo where inline tests get ejected into siblings and
e2e coverage spans three more.
