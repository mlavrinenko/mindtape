#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Estimate as a unit dict with per-project units",
  status: wip(2026, 6, 18)[shape change estimate-hours -> estimate dict; units carry factors],
  priority: "normal",
  estimate-hours: 4,
  tags: ("substrate", "cli"),
)

== Summary

Replace the scalar `estimate-hours: 1` with a unit dict `estimate: (hours: 1,)`.
Multi-unit estimates (`estimate: (days: 5, hours: 5)`) are allowed when the
project permits those units. A bare unit dict is not comparable, so each unit
carries a conversion factor to a common base; the project owns those factors.
MindTape ships sane defaults so an unconfigured project still normalizes and
validates.

== Decision

The prelude stays consumer-blind: it renames the core slot to `estimate` and
emits the dict verbatim, validating nothing about units. The Rust layer owns
unit policy via a dedicated `[tasks.estimate]` config table:

```toml
[tasks.estimate]
units = { hours = 1, days = 8, weeks = 40 }  # value = size in base units
required = "non-terminal"
min = 0
```

The base unit is the one whose factor is 1 (hours by default). The loader
normalizes an estimate to a single scalar `estimate_total = Σ qty × factor` in
base units and stores it alongside the raw unit map. Validation flags unknown
units, enforces `min`/`max` on the total, and honors required-unless-terminal.
Filter and sort expose `estimate` as that normalized total; JSON output emits
the raw `estimate` object plus `estimate_total`.

== Scope

- `lib/prelude.typ`: core slot `estimate-hours` -> `estimate`; emit the dict.
- `crates/mindtape`: `config` gains `[tasks.estimate]` (units/required/min/max,
  default unit table); `model` carries `estimate: BTreeMap<String, f64>` and
  `estimate_total: Option<f64>`; `loader` normalizes via config; `validate`
  checks units/bounds/required; `filter` field `estimate_hours` -> `estimate`.
- `crates/mindtape-cli`: `output` emits `estimate` + `estimate_total`; `list`
  sort field rename; `default-config.toml` ships `[tasks.estimate]`.
- Migrate every `truth/tasks/*.typ` and the doc examples (README, www) to the
  new shape. `docs/redesign/*` is frozen history; leave it.
- Regression tests across config parse, normalization, validation, filter.
