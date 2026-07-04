#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Clearer error when floor-inherited int bands collide with a real scale",
  status: wip(2026, 7, 4),
  priority: ice(impact: 4, confidence: 0.75, ease: 7),
  tags: ("substrate",),
  depends-on: (link("config-embedded-defaults.typ"),),
)

== Summary

With the embedded floor in place, a project that sets `[tasks.priority] kind =
"real"` but does not override `bands` inherits the floor's int-contiguous bands
(0-2 / 3-6 / 7-9). Under real semantics those bands are non-contiguous, so config
load fails with `bands `low` and `normal` are not contiguous` — a message that
names neither the cause (the bands came from the defaults floor, not the user's
file) nor the fix. A user who never wrote a `bands` line gets an error about
bands. Surfaced integrating the real-scale add-slot work against the floor.

== Scope

- Improve the band-contiguity error in `crates/mindtape/src/numeric.rs`
  (`NumericField::validate`, the contiguity check behind the message). When the
  scale is `Real` and the adjacent bands WOULD be contiguous under `Int`
  (`prev.max + 1 == next.min`, i.e. classic integer tiling), append actionable
  guidance, e.g.: "these bands tile an integer scale but `kind = real`; if they
  are inherited from the defaults floor, set `bands = []` to clear them or define
  bands that tile your real scale."
- Keep the existing message for genuinely-malformed real bands (an actual gap not
  explained by int tiling) — only add the floor hint for the int-tiling case.
- No config provenance needed: the int-tiling shape (`prev.max + 1 == next.min`
  for every adjacent pair) is a reliable, local signal that the bands are the
  shipped int defaults under a real override.

== Tests

- `kind = "real"` + floor-shaped int bands (0-2/3-6/7-9) → error includes the
  `bands = []` hint.
- `kind = "real"` + genuinely-gapped real bands (e.g. 0-1, 5-9) → original
  message, no misleading floor hint.
- `kind = "int"` + the same int bands → still valid (no regression).

== Notes

- Message-only change plus the detection heuristic; no behaviour change to what is
  accepted or rejected. Explicit error over magic auto-clearing: do NOT silently
  drop the floor bands when kind flips to real — tell the user, let them clear.
- Small, self-contained; touches only `numeric.rs` + its tests.
