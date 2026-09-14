#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Require a minimum word count on a status note",
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

`[[tasks.status]] note = "required"` only asks whether a note exists, so a closing note reading "done" satisfies it. Once the closing note is the outcome record, the write-once dated addendum that ADRs, Rust RFCs and the design-doc conventions all converge on, it has to say something. A word minimum is all the policing it needs: the note stays free-form text, with no pattern or content assert over it.

== Scope

Add one key to a legend entry:

```toml
[[tasks.status]]
value = "done"
note-min-words = 10
```

A flip into a status declaring `note-min-words` must carry a note of at least that many words, so the key implies a note on that flip. Words are counted the way content rules count them (whitespace-delimited tokens containing an alphanumeric character), by sharing `word_count` from `crates/mindtape/src/content_gate.rs` rather than copying it.

The key is enforced at the flip only, never by `mt check`: a note is written once, at the transition, and closed history is not rewritten. Measured on 2026-09-14, 141 of 331 closed tasks here carry no note and 54 carry fewer than 10 words, so a check-time rule would fail history nobody will edit. `note = "required"` keeps its current meaning, untouched.

Put the rule on `LegendEntry` in `crates/mindtape/src/config_status.rs`, red tests first in `config_status_tests.rs`: no note fails at 10, a three-word note fails, a ten-word note passes, and an entry without the key accepts anything. Call it from `check_note_required` in `crates/mindtape-cli/src/flip.rs`, and bless `schema/config.schema.json` in the same commit. The end-to-end pin, the docs and this repo's own minimum are `document-the-status-note-word-minimum.typ`.

== Dogfood

Give this repo's closing statuses a word minimum in `.mindtape/config.toml`, landed in `document-the-status-note-word-minimum.typ`.
