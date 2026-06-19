#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Task identity: explicit id, uniqueness, configurable format",
  status: done(2026, 6, 19),
  priority: "high",
  estimate: (days: 1),
  relates-to: (link("link-resolution-by-path.typ"), link("list-handle-ordinal-check.typ")),
  tags: ("substrate",),
)

== Summary

Identity is always the file stem today: `loader.rs` calls `file_stem(file)` and
never reads the `id:` marker slot the prelude already emits — the explicit-id
escape hatch is wired in the DSL but dead in Rust. Duplicate stems across nested
folders collide silently (`resolve_task` picks the first `id == stem` match) and
`check` never validates uniqueness. Fix the identity model so identity is a
stable, explicit, validated key.

== Scope

- Wire the dead slot: explicit `id:` wins, else the file stem. The loader reads
  `fields.id` instead of discarding it.
- Hard uniqueness check in `check`: fail on duplicate ids rather than silently
  resolving to the first match.
- Configurable id format under `[tasks.id]`: default is an arbitrary string (the
  current behaviour). Ship UUID formats out of the box (v4, v7). Validate that an
  explicit `id:` conforms to the configured format.
- A future `add` generates a conformant id for the configured format.

== Boundary

Identity is the addressing/handle key, not the file-path link key. `link()`
targets stay file paths (see link-resolution-by-path); the two must not be fused.
