#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Nest `mindtape add` into a subfolder under the root",
  status: done(2026, 6, 27),
  priority: ice(impact: 6, confidence: 0.8, ease: 8),
  relates-to: (
    link("add-template-opt-title-arg.typ"),
  ),
  tags: ("cli",),
)

== Summary

`mindtape add --to shopping Potatoes` should file `<root>/shopping/Potatoes.typ`
— nest a new task into a subfolder under the destination root. Today `add` only
writes flat into the root chosen by `--into` (default the first configured
root); reaching a nested folder means hand-spelling `--into <root>/shopping`,
which is root-relative-blind (it resolves against cwd) and clumsy.

The pay-off is folder-derived tags (`[tasks.derive-tags]`): nesting a task into
`shopping/` makes it inherit the `shopping` tag with no hand-wired `tags`.

== Scope

- New `--to <SUBDIR>` flag: a subfolder path relative to the destination base.
  The base stays whatever `--into` resolves (default `.` = first root), so
  `--to` composes on top: `--into . --to shopping` → `<first-root>/shopping`.
- Reject an absolute `--to` (it names a subfolder, not a base); a base override
  belongs in `--into`. Nested subpaths (`shopping/weekly`) are allowed.
- `create_dir_all` already builds missing parents; the existing
  `warn_if_outside_roots` check still fires when the nest escapes every root.
- `dest_dir` grows the `to` argument; the write/verify path is unchanged.

== Notes

- Touches `crates/mindtape-cli/src/add.rs` (`AddArgs`, `dest_dir`, `run`) only;
  no library churn.
- README `add` usage block gains a `--to` line; CLAUDE.md `add` note mentions the
  subfolder form alongside `--into .`.
