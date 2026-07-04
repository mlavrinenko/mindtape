#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Embedded defaults floor: generalise the framework deep-merge to the whole config",
  status: wip(2026, 7, 4),
  priority: ice(impact: 8, confidence: 0.8, ease: 5),
  tags: ("substrate",),
)

== Summary

Config has two divergent "default" sources: the Rust `Default` impls (what an
omitted section resolves to) and the opinionated `mt init` scaffold
(`crates/mindtape-cli/src/default-config.toml`). They drift. The omitted-legend
bug (`Legend::default()` is empty, so `Legend::terminal()` returns nothing and
required-unless-terminal / bare `mt rm` / overdue accent all misbehave) is one
instance of the disease.

Fix: make a single embedded TOML floor the source of truth, deep-merged under the
user config before typed deserialisation — generalising the existing
`merge_builtin_frameworks` precedent (`crates/mindtape/src/framework.rs:428`) from
the priority-frameworks sub-table to the entire config document.

Supersedes and closes `default-status-legend.typ` (its option A).

== Scope

- New `crates/mindtape/src/defaults.toml` — the canonical default VALUES (not the
  commented init scaffold). Embed via `const EMBEDDED_DEFAULTS: &str =
  include_str!("defaults.toml")` in `config.rs`, mirroring `BUILTIN_FRAMEWORKS`
  (`framework.rs:27`). Stage the new file with `git add` before any `nix`/build
  step (flake eval ignores untracked files).
- Extract `deep_merge` (currently private in `framework.rs:440`) into a shared
  `crates/mindtape/src/merge.rs` (`pub(crate) fn deep_merge`), used by both the
  frameworks merge and the new defaults merge. No behaviour change to `deep_merge`
  itself (tables merge field-by-field; arrays and scalars replace wholesale).
- `parse_config` (`config.rs:436`): build the doc as
  `EMBEDDED_DEFAULTS` (base) → `deep_merge(user text)` (overlay) →
  `merge_builtin_frameworks` → `try_into::<Config>()`. Order matters: defaults
  first, user on top, frameworks last (unchanged).
- `load_config` no-file branch (`config.rs:448-449`): return `parse_config("")`
  instead of `Config::default()`, so a project with no `config.toml` inherits the
  floor.
- Collapse the opinionated Rust `Default` impls into the floor so there is one
  source of truth. Move into `defaults.toml`, simplify the impls to neutral
  empties: `RmConfig::default` `Some("terminal")` → `None` (`config.rs:336`);
  `TasksConfig::default` `roots=["tasks"]` (`config.rs:134`); `TemplatesConfig`
  dirs/default (`config.rs:74`); `NumericField` 0..9 scale (`numeric.rs:167`).
  `Config::default()` becomes the truly-empty config (used only by tests /
  programmatic callers); the floor is applied only through `parse_config`.

== Floor contents (the reviewable decision)

Include in `defaults.toml`:
- The 6-value status legend: `proposed`/`accepted`/`wip` (non-terminal),
  `done`/`cancelled`/`failed` (`terminal = true`), with the glyphs/colours from
  `default-config.toml:28-59`. This is the one that fixes terminal semantics.
- `[tasks.priority]` `kind=int, min=0, max=9`, the three bands (low/normal/high).
- `[tasks.difficulty]` `kind=int, min=0, max=9`, the four anchors.
- `roots = ["tasks"]`; `[templates]` dirs/default; `[rm] default-filter = "terminal"`;
  `[ls] default-filter = "!terminal && (miss(start) || start <= now())"`.

Exclude (stay unconstrained when omitted, preserving the white-list where it is
safe and vocabulary is project-specific): `[tasks.estimate] units`, `[tasks.tags]
allowed`, `[tasks.body.headers]`, `[tasks.derive-tags]`.

Semantic shift to document: "omit a section" now means "inherit the floor", not
"unconstrained". A project explicitly unconstrains a floored section by setting it
empty — `status = []`, `[tasks.priority] bands = []` — which the array-replace
merge honours. Add this to `defaults.toml`'s header comment and the CLAUDE.md /
README white-list prose.

== Tests

- Omitted legend → `parse_config("").tasks.status.terminal()` == `[done,
  cancelled, failed]`.
- Explicit `[[tasks.status]]` in user text fully replaces the floor legend
  (array-replace), not merges.
- `status = []` clears the floor legend to empty.
- Frameworks still merge (existing `framework_tests` stay green); a user
  framework override still wins field-by-field.
- `load_config` on a dir with no `config.toml` equals `parse_config("")`.
- Regression: the dogfood config and `default-config.toml` still parse and
  validate unchanged.

== Notes

- Heaviest task in this set; unblocks the discovery layer, `mt config show`, slim
  init, and the add-initial-status fix. Land first.
- `deny_unknown_fields` is enforced at deserialise time, after the merge, so the
  floor and user docs share one schema — no special-casing needed.
- Keep `defaults.toml` under the library crate, not the CLI: the floor is a
  domain concern (`mindtape`), consumed by any tool on the library, while the
  init scaffold stays CLI-side.
