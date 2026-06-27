#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rename `[query]` config section to `[ls]`",
  status: done(2026, 6, 23),
  priority: ice(impact: 5, confidence: 0.95, ease: 8),
  depends-on: (link("filter-now-date-function.typ"),),
  tags: ("cli",),
)

== Summary

The `[query]` section only drives `mindtape ls` defaults; `[rm]` is the
parallel section for `mindtape rm`. Renaming to `[ls]` makes the two
sections consistent and the config self-documenting.

== Scope

- `crates/mindtape/src/config.rs`: rename `QueryConfig` → `LsConfig`,
  `Config.query` → `Config.ls`, update validation error messages.
- `crates/mindtape-cli/src/ls.rs`: `config.query.*` → `config.ls.*`.
- `crates/mindtape-cli/src/util.rs`: same.
- `crates/mindtape/src/sort.rs`: `config.query.default_sort` → `config.ls.default_sort`.
- `crates/mindtape-cli/src/default-config.toml`: `[query]` → `[ls]`.
- `.mindtape/config.toml`: `[query]` → `[ls]`; update `default_filter`
  to `"!terminal && (miss(start) || start <= now())"` (requires `now()`
  from `filter-now-date-function.typ`).
- `docs/redesign/config.example.toml` if it references `[query]`.

== Notes

- This is a breaking config change; acceptable on pre-release.
- No serde `rename` compatibility shim needed — just rename.
