#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "derived tags from folder structure",
  status: done(2026, 6, 20),
  priority: ice(impact: 7, confidence: 0.8, ease: 5),
  tags: ("substrate", "cli"),
)

== Summary

Tasks nested under a root inherit tags from their folder structure, so deep
trees stop demanding hand-wired tags. For a file under a derivation-enabled
root, MindTape injects virtual tags from the root name plus the folder segments
between the root and the file. Tags are computed every eval, never written to
the file. Authored `tags` stay pristine; derived tags live alongside.

Goal: simplify tag management for heavily nested backlogs.

== Scope

Engine in `crates/mindtape` (reusable; speconaut converges onto it), CLI thin.

- config: promote `tasks.roots` to `Vec<RootSpec>`, an untagged enum of a string
  shorthand or a `RootConfig { path, name, derive-tags }`. `name` defaults to the
  root path's last segment and is always emitted as a tag when derivation runs.
  `derive-tags` is `bool | "glob" | ["glob", …]`: gates which files under the
  root get derivation, overrides the global enable. New `[tasks.derive-tags]`
  block: `enabled` (project-wide default), `collect`, `separator`, `slugify`,
  `breadcrumb-root`.
- collectors (`collect`, both may run together, union + dedupe):
  - `segments`: each folder name an independent flat tag (`backend`, `auth`).
  - `breadcrumbs`: cumulative path prefixes joined by `separator` (default `:`):
    `backend`, `backend:auth`. `breadcrumb-root = true` folds the root name into
    the chain (`mt`, `mt:backend`, …); default off (root name stays standalone).
- `slugify` (default on, knob to disable): normalise folder names to tag form.
- new pure module `crates/mindtape/src/derive.rs`:
  `(root_cfg, rel_path_within_root) -> Vec<String>`. No IO, unit-testable.
- loader: thread the resolved root + path-relative-to-root into `load_file`,
  inject derived tags after `from_marker`. File outside the glob gets nothing
  (root name is part of derivation).
- model: keep `Task.tags` authored-only; add `Task.derived_tags` plus a deduped
  union accessor.
- filter: `has_tag` / `has(tags)` read the union.
- validate: derived tags validated against `tasks.tags.allowed` strictly, same
  as authored, with a distinct message (`derived from folder`). `allowed` stays
  literal-only for v1.
- output: json exposes `derived_tags` separate from `tags`.
- add: `roots.first()` and the outside-roots warning read `RootSpec.path`.
- docs: README config examples, CLAUDE.md config + DSL sections, `www`.
- dogfood: switch `.mindtape/config.toml` root to `name = "mt"`, derivation on,
  add `mt` to the tag allow-list.
