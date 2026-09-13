#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Let a status note declare a content policy",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 5,
  tags: ("mt", "cli", "gating"),
  estimate: (files: 7),
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("fix-the-status-note-flattening-on-read.typ"),
)

== Summary

`[[tasks.status]] note = "required"` is the only thing a project can say about a note, so a closing note reading "done" satisfies it. Once the note is the artifacts outcome record — the write-once dated addendum that ADRs, Rust RFCs and the design-doc conventions all converge on — its quality is what needs policing, and the vocabulary for that already exists as ContentAssert.

== Scope

Widen the legends NoteRule to accept a table reusing ContentAssert alongside the existing "required" string, so `note = { required = true, min-words = 10, must-not-match = "..." }` parses. Enforce it in crates/mindtape-cli/src/flip.rs check_note_required and in mt check, with a red test for each seam.
