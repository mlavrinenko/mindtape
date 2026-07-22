#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "status note rule",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 7,
  ),
  tags: (),
  status: proposed(2026, 7, 22),
)

== Summary

Let a project require a written note on a status stamp, per status value. Today `[[tasks.status]]` (LegendEntry in crates/mindtape/src/config_status.rs) carries value/glyph/color/label/terminal/to and nothing about the note; the note on `wip(2026, 7, 22)[why]` is always optional. Typed links already have the exact primitive to copy: LinkKind carries a NoteRule of Required/Optional/Forbidden, and the built-in `related` kind defaults to Required so the escape-hatch edge must justify itself. Mirror that onto the status legend so a preset can demand a justification on every transition — the motivating case is an AI-agent preset where an unexplained flip to done is the failure mode.

== Scope

- Add `note` (NoteRule) to LegendEntry, defaulting to Optional so every existing config is unchanged.
- Enforce in validate: a task whose current status value declares note = required and carries a blank/absent note is a validation error naming the field.
- Gate `mt flip` on it: refuse a transition into a note-required status without a note, and point at the flag that supplies one.
- Decide whether the rule is per-entry only or also settable once for the whole legend.
- Surface in the JSON schema and `mt init --annotate` commentary.
- Tests: required-note status rejects a blank note, accepts a filled one, forbidden/optional unchanged, and flip refuses then accepts.
