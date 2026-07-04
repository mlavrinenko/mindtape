#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Layered defaults discovery: walk-up `defaults.toml` + XDG under the project config",
  status: done(2026, 7, 4),
  priority: ice(impact: 7, confidence: 0.75, ease: 5),
  tags: ("substrate",),
  depends-on: (link("config-embedded-defaults.typ"),),
)

== Summary

With the embedded floor in place, add user- and directory-scoped default layers so
a user gets zero-setup defaults, an optional global app config, and directory-tree
defaults (editorconfig / direnv style) — while a project still fully overrides via
its own `config.toml`. Multiple `defaults.toml` may contribute; the nearest wins.

== Merge chain (lowest → highest precedence)

1. `EMBEDDED_DEFAULTS` (compiled floor, from the foundation task).
2. XDG global: `dirs::config_dir()/mindtape/defaults.toml` (Linux
   `~/.config/mindtape/defaults.toml`), if it exists.
3. Directory-scoped: every `.mindtape/defaults.toml` found walking UP from the
   project root, merged farthest-ancestor first so the NEAREST file wins.
4. Project: `<root>/.mindtape/config.toml` (highest), if it exists.

Then `merge_builtin_frameworks`, `try_into`, `validate` (unchanged tail).

== Scope

- New discovery module (e.g. `crates/mindtape/src/config_layers.rs`). Keep IO
  (locating files) separate from merging (pure) for testability:
  - `discover_layers(root, home) -> Vec<Layer>` where
    `Layer { source: LayerSource, text: String }` in precedence order. `LayerSource`
    is an enum: `Embedded`, `Xdg(PathBuf)`, `Dir(PathBuf)`, `Project(PathBuf)`.
  - `merge_layers(&[Layer]) -> Result<toml::Table, ConfigError>` — fold with
    `deep_merge` (shared from the foundation task's `merge.rs`).
- Walk-up: reuse the `find_root` pattern (`loader.rs:31`) but collect ALL
  ancestor `.mindtape/defaults.toml`, not just the first `.mindtape/` dir. Start
  at the project root, ascend. Boundary: stop after `$HOME` inclusive
  (`dirs::home_dir()`); if the root is not under `$HOME`, stop at the filesystem
  root. Never ascend above `$HOME`.
- Rework `load_config(root)` to assemble the chain via `discover_layers` +
  `merge_layers`, replacing today's single-file read (`config.rs:446`).
  `parse_config(text)` stays as the embedded-floor-plus-one-string entry (used by
  tests and the foundation task); the layered path is the new `load_config`.
- Return provenance for `mt config show --origin` (next task): add
  `load_config_layered(root) -> Result<(Config, Vec<LayerSource>), ConfigError>`
  and keep `load_config` as the `.0`-returning convenience.

== Tests

- Pure `merge_layers`: nearest `Dir` layer overrides a farther one; project
  `config.toml` overrides all defaults; XDG overrides embedded but loses to a
  `Dir` layer.
- `discover_layers` on a tempdir tree (home / parent-with-defaults /
  project-with-defaults+config): correct set and order; the `$HOME` boundary is
  respected (a `defaults.toml` above `$HOME` is not read).
- Missing XDG / missing directory defaults are skipped silently.
- No files anywhere → identical to `parse_config("")`.

== Notes

- XDG is intentionally LOWER precedence than a directory `defaults.toml`: a
  monorepo's tree-local defaults are more specific than a user's global taste.
- Directory `defaults.toml` vs the project `config.toml` split is the answer to
  "share `[ls]`/`[rm]`/`[add]`/`[go]`/`[typst]` across projects, override per
  project": put shared ergonomics in a defaults layer, keep the per-repo task
  model in `config.toml`. No schema partition needed — every key is legal in
  every layer; precedence does the work.
- Canonicalise paths before de-duping so a symlinked home does not double-count a
  layer.
