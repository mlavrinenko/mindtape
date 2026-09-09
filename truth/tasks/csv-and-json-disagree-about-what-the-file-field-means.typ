#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "csv and json disagree about what the file field means",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (files: 3),
  assignee: "agent",
  status: done(2026, 9, 9),
  links: related(
    "the-file-filter-field-and-the-file-column-name-different-paths.typ",
  )[the same file/path split, one layer down],
)

== Summary

`file` still names two values, one layer down from the filter field that just
became `path`. `--format json` reports the project-root-relative path
(`CanonicalTask`, `crates/mindtape/src/canonical.rs`); `--format csv` reports
the working-directory-relative one, because its cell goes through the same
`render::file_path::cell_text` the table column does
(`crates/mindtape-cli/src/output.rs`, `csv_value`). Two machine formats over
one field, disagreeing by cwd.

== Scope

Reproduce from a subdirectory of the root — `cd tasks && mt ls '' --format
json` prints `"file": "tasks/a.typ"` while `mt ls '' --columns file --format
csv` prints `../tasks/a.typ` for the same task.

Neither is obviously wrong on its own, which is why this is a decision and
not a bug fix:

- JSON is a schema (`www/content/schema.typ` documents `file` as computed,
  and `mt import` reads the same shape back), so its value must not depend on
  where the producer stood. Root-relative is the only defensible choice, and
  it should not move.
- CSV is pinned to the table by `path_format_matches_the_file_column_exactly`
  (`crates/mindtape-cli/tests/ls_path_e2e.rs`) — a deliberate pairing, since
  `--format path` exists to be pasted into another command from the cwd that
  produced it. Making CSV root-relative breaks that pairing and the test that
  states it.

So pick one:

- declare CSV a display format (it renders whatever the resolved columns
  render) and say so where the `file` column is documented, leaving the
  divergence but removing the surprise;
- give CSV both columns — `file` root-relative for the record, and leave the
  cwd-relative form to `--format path`, whose whole job it is;
- rename per surface the way the filter field just was, so no two surfaces
  share a name for two values.

The third is the same move as
`truth/tasks/the-file-filter-field-and-the-file-column-name-different-paths.typ`
and the most consistent, but it costs a schema break on the JSON side, which
the first bullet above argues against. Reading order matters here: settle
whether CSV is a record format or a rendering first, and the rest follows.

= Decision

CSV is a record format, and its `file` cell is now the root-relative path
`--format json` already reported — the second bullet above, minus the extra
column, since one value under one name needs no second column.

What settled it was already written down twice. `www/content/cli.typ` calls
`csv` a data contract where it argues why the `file` column auto-shows off a
terminal for `table` only, and `render_csv`'s own doc comment declares CSV
values to be the plain field rather than the table's cell text — `file` was
the one column not obeying the rule its renderer states. The `#` column had
made the same move for the same reason: the table's handle is ephemeral per
render, so CSV emits the stable `id` instead. A cwd-relative path is ephemeral
in exactly that way, so `file` follows `#`.

The pairing `path_format_matches_the_file_column_exactly` protected is intact
and now reads the column it was always about: `--format path` and the table
cell, the two surfaces a human reads from a shell. Only CSV left that pair,
and the test asserts it from a subdirectory now, where the claim has content.

= Watch for

Whatever lands, `contrib/jq/*.jq` and their fixtures read the JSON `file`
key, and `www/content/import-export.typ` states that `mt import` ignores
`file` on the way back in. Does any of that change meaning?

Neither does. Nothing about the JSON side moved — `contrib/jq/*.jq` select on
the same key over the same value, and `mt import` still ignores `file` as a
computed field. What gained meaning is a CSV export: its `file` column is now
a value the `path` filter can be built from.
