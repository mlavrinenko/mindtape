#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Error on unknown config fields",
  status: done(2026, 6, 27),
  priority: ice(impact: 7, confidence: 0.95, ease: 8),
  tags: ("cli",),
)

== Summary

Serde ignores unknown config fields by default, so a typo like `framewokr = "ice"`
in `.mindtape/config.toml` is silently dropped and the user never notices. Add
`#[serde(deny_unknown_fields)]` to all deserialised config structs so mistyped or
stale keys are reported as a parse error. Serde's `unknown_field` error already
includes the list of expected field names, so suggestions come for free.

== Scope

- Add `deny_unknown_fields` to each config struct that can safely receive it:
  structs without `#[serde(flatten)]` fields and without `#[serde(transparent)]`.
  Files: `crates/mindtape/src/config.rs`, `config_roots.rs`, `config_add.rs`,
  `body_headers.rs`, `numeric.rs`, `framework.rs`.
- Skip `PriorityConfig` (carries `#[serde(flatten)] pub field: NumericField`):
  the `deny_unknown_fields` + `flatten` combination is unsupported by serde.
  `NumericField` itself carries the attribute; the restriction is only on the
  container that declares the `flatten` field.
- Skip `Legend` (`#[serde(transparent)]`); untagged enums (`RootSpec`, `DeriveTags`)
  and unit enums (`Collector`, `InteractiveMode`, `InputKind`, `ScaleKind`).
- Error message from serde is already helpful, e.g.:
  "unknown field `framewokr`, expected one of `framework`, `allow-manual`, `frameworks`"
  No extra suggestion logic needed — serde supplies it.

== Notes

- `deny_unknown_fields` is compatible with `#[serde(default)]`: it rejects unknown
  keys but still allows all declared fields to be omitted.
- Remaining gap: `[tasks.priority]` outer-level typos (wrong spelling of
  `framework`, `allow-manual`, `frameworks`) are still silently ignored because
  `PriorityConfig` cannot have `deny_unknown_fields`. This is a narrow, known gap;
  fixing it would require unwrapping the `flatten` and duplicating `NumericField`'s
  fields into `PriorityConfig`.
