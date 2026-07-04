#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Synthetic add slots honour `ScaleKind::Real` instead of always `kind: \"int\"`",
  status: done(2026, 7, 4),
  priority: ice(impact: 3, confidence: 0.8, ease: 8),
  tags: ("substrate",),
)

== Summary

`required_core_slots` synthesises the priority / difficulty scaffold slots with a
hardcoded `kind: "int"` (`crates/mindtape/src/add_slots.rs:44,73`), ignoring the
field's configured `ScaleKind`. A project with `[tasks.priority] kind = "real"`
gets a mistyped synthetic slot in `mt add --describe`, the scaffold, and the
written task.

== Scope

- In `add_slots.rs`, read the field's `ScaleKind` (`numeric.rs:98`,
  `NumericField.kind`) and emit `kind: "real"` when it is `Real`, `"int"` when
  `Int`, for both the priority slot (line 44) and difficulty slot (line 73).
- Confirm the `SlotKind` / template fill path accepts a `real`-kind value and
  writes a float literal without truncation (`template.rs` fill; `discover_slots`
  already walks nested calls).

== Tests

- `required_core_slots` for a `real`-kind priority field yields a slot whose kind
  is `real`.
- `mt add --describe=json` for a real-scale project reports `real` for that slot.
- A filled real value (`--set priority=2.5`) round-trips into the written task and
  passes `mt check`.

== Notes

- Small, self-contained; no dependency on the config-layering tasks.
- The estimate-slot hint fallback unit `"days"` (`add_slots.rs:92`) is cosmetic
  and out of scope here — leave it.
