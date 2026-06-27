#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Unify config key case on kebab-case",
  status: accepted(2026, 6, 27),
  priority: ice(impact: 4, confidence: 0.95, ease: 8),
  tags: ("cli",),
)

== Summary

Multi-word config keys mix conventions. Six are kebab (`local-packages`,
`derive-tags`, `allow-manual`, `date-format`, `breadcrumb-root`); the `[ls]` /
`[rm]` `default_filter` and `[ls] default_sort` are snake — leftovers from the
`[query]` era. Within one `[ls]` block a user writes `date-format` next to
`default_filter`. Pick one convention before 0.2 ships. Kebab: dominant here,
matches the Typst DSL idiom (`depends-on`, `x-<ns>`) and the Cargo/TOML
ecosystem, fewer keystrokes.

== Scope

- Switch the case bridge from per-field `#[serde(rename = "...")]` to
  container `#[serde(rename_all = "kebab-case")]` on each config table struct,
  so the convention is structural and future multi-word fields are kebab by
  construction (this inconsistency arose because a field was added without a
  per-field rename). Affected structs: `TypstConfig`, `TasksConfig`,
  `PriorityConfig`, `LsConfig`, `RmConfig` (`crates/mindtape/src/config.rs`);
  `RootConfig`, `DeriveConfig` (`crates/mindtape/src/config_roots.rs`). Remove
  the now-redundant per-field renames.
- `PriorityConfig` flattens `NumericField`; container `rename_all` does not
  reach flattened keys, but `NumericField`'s keys are all single-word, so no
  rename_all is needed there.
- Net user-facing change: `default_filter` → `default-filter` (both `[ls]` and
  `[rm]`), `default_sort` → `default-sort`. The six already-kebab keys are
  unchanged.
- Update real config + docs: `.mindtape/config.toml`,
  `crates/mindtape-cli/src/default-config.toml`, `README.md`,
  `docs/redesign/config.example.toml`, and any `www/` config example.
- Update TOML strings in tests (`config_tests.rs`, `sort_tests.rs`) and the
  validation error path `ls.default_sort` → `ls.default-sort`
  (`crates/mindtape/src/config.rs`). Rust field names stay snake (idiomatic);
  only serialized keys and user-facing messages change.

== Notes

- Breaking config change; acceptable on pre-release, no compatibility shim.
- Out of scope (not case issues, keep as-is): `rename = "type"` (reserved
  word), `rename = "category"` (singular array-of-tables vs plural `Vec`), and
  the `lowercase` enum-value renames.
