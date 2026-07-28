#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Filter and sort tasks by file",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 4),
  status: proposed(2026, 7, 28),
)

== Summary

`contains(file, "shopping/")` and `--sort file` do not work: `file` is absent
from the filter language's field dispatch and from `SORT_FIELDS`. A project
that encodes meaning in its folder structure — the same structure
`[tasks.derive-tags]` already mines — has no way to slice on it beyond derived
tags, and those only surface the segments a root opted into. Exposing the path
itself is the general escape hatch, and it costs two match arms.

== Decisions

- Both the filter value and the sort key read the root-relative path, not the
  working-directory-relative one the `file` column displays. A filter lives in
  `.mindtape/config.toml` or a `[views.*]` bundle; if it read the cwd-relative
  form, one view would match different tasks depending on which directory `mt`
  ran from. Display is localized, the stored value is not — the same split
  dates already have between the ISO value and `[ls] date-format`.
- The value uses `/` as separator on every platform, so an expression written
  on one machine is portable to another.
- No new operators. `file` is a plain string field, so `==`, `!=`, and the
  existing `contains(field, "substr")` already cover it.

== Scope

- `field_value` in `filter.rs` gains a `"file"` arm yielding the root-relative
  path as a string.
- `SORT_FIELDS` gains `"file"`, with the same string as the sort key.
- Tests: `contains(file, "sub/")` against a nested task; equality against a
  full path; sort ordering across a mixed set; separator normalisation.
- Out of bounds: no edits under `www/`, `skills/`, `README.md`, `AGENTS.md`, or
  any render or config file. Documentation lands in one reconcile pass owned by
  hyperlink-the-file-column-with-osc-8.
