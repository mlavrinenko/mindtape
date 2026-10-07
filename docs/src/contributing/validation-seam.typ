// docs/src/contributing/validation-seam.typ renders
// docs/contributing/validation-seam.md through docs/generate-md.sh (typlite;
// docs/src/lib.typ lists the constraints).
//
// AGENTS.md links here from its Task DSL section, for a reader adding a
// validation.
#import "../lib.typ": doc-title

#metadata((
  output: "docs/contributing/validation-seam.md",
)) <mindtape-md>

#doc-title[Where validation lives]

Which layer a new check on a task belongs in:

- The prelude checks structure only: one task per file, the `x-<ns>` field
  names, and the argument shapes it refuses before it writes the marker (edge
  targets and notes, a status note, framework inputs). Rust cannot report
  these mistakes, because a file that makes them produces no marker.
- Rust checks everything that reaches the marker: the status legend, the
  priority and difficulty scales, the priority frameworks and their scoring,
  the edge kinds and their rules, the `required` rules (a filter expression
  per field), bounds, and link resolution. It checks types too.
  `crates/mindtape/src/model/marker.rs` reports a field of the wrong shape, and a
  wrong-typed element inside a field of the right shape, with its position
  (`tags[1]`, `estimate.days`, `recur.on[1]`).
- The card render checks nothing. Every accessor checks its value's type, and
  `_scalar` falls back to `repr`, so a wrong-typed value reaches the marker
  and Rust reports it. An eval error in the render would name neither the
  field nor the task.
- A consumer validates its own `x-<ns>` fields. MindTape passes them through
  unchanged.

So a new check on a value goes in Rust and config, and a new check on the
call's shape goes in the prelude. A check in the other layer never fires.
