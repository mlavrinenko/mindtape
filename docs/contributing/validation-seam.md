<!-- Generated from docs/src/contributing/validation-seam.typ by `just docs md`. Do not edit; edit the source. -->

# Where validation belongs

Four layers can validate a task. A check in the wrong one never runs:

- The prelude checks one task per file, the `x-<ns>` field names, and the argument shapes it refuses before it writes the marker (edge targets and notes, a status note, framework inputs). Rust can’t report these mistakes, because a file that makes them doesn’t produce a marker.
- Rust checks everything that reaches the marker: the status legend, the priority and difficulty scales, the priority frameworks and their scoring, the edge kinds and their rules. It also checks the `required` rules (a filter expression per field), bounds, and link resolution. It checks types too. `crates/mindtape/src/model/marker.rs` reports a field of the wrong shape, and a wrong-typed element inside a field of the right shape, with its position (`tags[1]`, `estimate.days`, `recur.on[1]`).
- The card render doesn’t check anything. Every accessor checks its value’s type, and `_scalar` falls back to `repr`. A wrong-typed value reaches the marker, and Rust reports it. An eval error in the render would name neither the field nor the task.
- A consumer validates its own `x-<ns>` fields. Mindtape passes them through unchanged.

A check on a value goes in Rust and config. A check on how you write the call goes in the prelude.
