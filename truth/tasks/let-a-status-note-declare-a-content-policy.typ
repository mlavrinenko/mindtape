#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Let a status note declare a content policy",
  kind: "feat",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("fix-the-status-note-flattening-on-read.typ"),
)

== Summary

`[[tasks.status]] note = "required"` is the only thing a project can say about a note, so a closing note reading "done" satisfies it. Once the note is the artifacts outcome record — the write-once dated addendum that ADRs, Rust RFCs and the design-doc conventions all converge on — its quality is what needs policing, and the vocabulary for that already exists as ContentAssert.

== Scope

Widen the legend's note rule to accept a table reusing `ContentAssert` alongside the existing `"required"` string:

```toml
[[tasks.status]]
value = "done"
note = { required = true, min-words = 10, must-not-match = "^done$" }
```

The legend entry uses `mindtape::link::NoteRule` today, shared with `links:` edge kinds, so the status side needs its own type rather than widening the edge one. Touches `crates/mindtape/src/config_status.rs` and `config_status_tests.rs` for the type, `crates/mindtape/src/content_gate.rs` to expose the assert evaluation over a bare string rather than a section, and the `mt check` seam in `crates/mindtape/src/validate.rs` with a red test in `validate_tests.rs`. The `mt flip` seam and the docs follow in `enforce-a-status-note-content-policy-on-flip.typ`.
