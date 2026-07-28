#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Opt-in file column for mt ls",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 8),
  status: done(2026, 7, 28),
)

== Summary

`mt ls` gives you no way to get from a listed row to the file behind it. The
`#` handle is ephemeral — an ordinal recomputed on every render — and the id is
a bare stem, so reaching the source means reconstructing the path by hand.

Add a `file` column carrying the task's source path relative to the process
working directory. That form is the one an IDE-integrated terminal (Zed, VS
Code, JetBrains) already detects and turns into a ctrl-click that opens the
file, with no escape sequences involved. The plain text is the feature.

== Decisions

- The column key and header are `file`, not `path`: one vocabulary across
  `Task.file`, `CanonicalTask.file`, the config, the filter language, and the
  column.
- It never renders by default. `file` joins `LS_COLUMNS` and the built-in
  render order (last, after `title`), but its `auto` visibility resolves to
  visible only when the column was named explicitly — via `--columns`, `[ls]
  columns-order`, a view's `columns`, or an `[[ls.rules]]` columns override.
  `[ls.columns.file] visibility = "visible"` forces it on under the built-in
  order; `"hidden"` always wins. The ordinary auto rule (non-empty means
  significant) cannot be reused: the cell is never blank, so it would pin the
  column permanently visible and there would be no opt-in at all.
- Cell text is working-directory-relative: `truth/tasks/x.typ` from the project
  root, `../truth/tasks/x.typ` from a subdirectory. Always relative; absolute
  only when no relative path exists.
- CSV renders it too, off the same resolved column set. JSON is untouched:
  `CanonicalTask.file` stays root-relative because it is interchange, not
  display.

== Scope

- `LS_COLUMNS` gains `file`; a `ColumnId::File` with header `FILE`, non-flex.
- `resolve_order` learns whether the order was explicitly listed, and feeds
  that in as the file column's significance.
- A pure `relativize(root, cwd, file)` helper on the CLI side, hand-rolled, no
  new dependency. The process-global `(root, cwd)` pair is recorded by
  `util::open()`, mirroring the existing `WORKTREE` and `COLOR_ENABLED`
  one-shot cells; the root-relative path is the fallback when it was never set,
  which is what unit tests see.
- `render_csv`, `csv_header`, and `csv_value` gain the column.
- Tests: relativize across same directory, subdirectory, sibling, and the root
  itself; hidden by default; shown when named; `visibility` overrides both
  ways; CSV column presence.
- Out of bounds: no edits under `www/`, `skills/`, `README.md`, or `AGENTS.md`.
  Documentation lands in one reconcile pass owned by
  hyperlink-the-file-column-with-osc-8, which is the task whose `cli.rs` edit
  trips the `cli-docs` drift group.
