#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Estimate as a unit dict with per-project units",
  status: done(2026, 6, 18)[estimate is an opaque unit dict; units are a per-project allow-list, no normalization],
  priority: "normal",
  estimate: (hours: 4),
  tags: ("substrate", "cli"),
)

== Summary

Replace the scalar `estimate-hours: 1` with a unit dict `estimate: (hours: 1,)`.
Multi-unit estimates (`estimate: (days: 5, hours: 5)`) and non-time units
(`estimate: (points: 8)`) are allowed when the project permits those unit names.
Units are opaque free labels: never converted, summed, or compared. The project
owns which unit names are legal; the shipped scaffold lists sane defaults.

== Decision

The prelude stays consumer-blind: it renames the core slot to `estimate` and
emits the dict verbatim, validating nothing about units. The Rust layer owns
the unit allow-list via a dedicated `[tasks.estimate]` config table:

```toml
[tasks.estimate]
units = ["minutes", "hours", "days", "weeks"]  # free labels; omit = any unit
required = "non-terminal"
```

No normalization: an estimate is stored as its raw `unit -> quantity` map.
Validation flags units outside the allow-list and honors required-unless-terminal.
There is one source of truth for the default units — the scaffolded
`default-config.toml`; the in-code default is empty (any unit), matching the
white-list philosophy that an omitted section is unconstrained. Filter/sort drop
numeric estimate support (a mixed-unit dict has no scalar); `has(estimate)` /
`miss(estimate)` cover presence. JSON output emits the raw `estimate` object.

== Scope

- `lib/prelude.typ`: core slot `estimate-hours` -> `estimate`; emit the dict.
- `crates/mindtape`: `config` gains `[tasks.estimate]` (units allow-list +
  required; empty default); `model` carries `estimate: BTreeMap<String, f64>`;
  `validate` checks the unit allow-list + required; `filter` makes `estimate`
  presence-only and drops the now-unused numeric `FieldValue` variant.
- `crates/mindtape-cli`: `output` emits the raw `estimate` object; `list` drops
  `estimate` as a sort field; `default-config.toml` ships `[tasks.estimate]`.
- Prune `FieldConfig` `type`/`min`/`max` (dead since estimate left the generic
  number-field path).
- Migrate every `truth/tasks/*.typ` and the doc examples (README, www) to the
  new shape. `docs/redesign/*` is frozen history; leave it.
- Regression tests across config parse, validation, filter presence.
