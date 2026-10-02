// docs/src/contributing/validation-seam.typ — source for
// docs/contributing/validation-seam.md. Rendered by docs/generate-md.sh via
// typlite, same as every other docs/src/*.typ source; see docs/src/lib.typ
// for the render path's constraints.
//
// Deep dive behind AGENTS.md's Task DSL section: the seam that decides which
// layer a new validation constraint belongs in. On the always-loaded surface
// it was a subsection every agent carried to answer a question it meets once;
// here the reader reaches it deliberately when adding a constraint.
#import "../lib.typ": doc-title

#metadata((
  output: "docs/contributing/validation-seam.md",
)) <mindtape-md>

#doc-title[Where validation lives]

The seam that decides which layer a change belongs in:

- The prelude enforces structure only — one task, the `x-<ns>` discipline, and
  the argument shapes it rejects before the marker is written (edge
  targets and notes, a status note, framework inputs). Those are the mistakes the
  Rust layer can never see, because nothing reaches it to validate.
- Everything that reaches the marker is the Rust layer's: the status legend,
  the numeric priority/difficulty scales and classification, priority frameworks
  (scoring, in-bounds by construction), the edge-kind enum and its per-kind rules,
  the `required` rules (a filter expression per field), bounds, and link
  resolution. Types too — `crates/mindtape/src/marker.rs` reports a wrong-shaped
  field, and any wrong-typed element inside a well-shaped one, naming the
  position (`tags[1]`, `estimate.days`, `recur.on[1]`).
- The card render enforces nothing. It is total by construction — every accessor
  type-guards, `_scalar` falls back to `repr` — so a wrong-typed value reaches
  the marker instead of failing eval on a method it does not have. An eval error
  there names neither the field nor the task, and disappears the moment the
  render stops making that call.
- Consumers validate their own `x-<ns>` fields. MindTape carries them through
  untouched.

So: a new constraint on a value goes in Rust and config. A new constraint on the
_call shape_ goes in the prelude. Putting either in the other place produces a
rule that cannot fire.
