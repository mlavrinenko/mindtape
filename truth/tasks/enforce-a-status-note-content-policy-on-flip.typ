#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Enforce a status note content policy on flip",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("mt", "cli"),
  estimate: (files: 7),
  kind: "feat",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("let-a-status-note-declare-a-content-policy.typ"),
)

== Summary

`let-a-status-note-declare-a-content-policy.typ` makes `mt check` police a status note's content, but `mt flip` still only asks whether a note is present, so a flip writes a note that the very next `mt check` rejects.


== Scope

Replace the presence test in `check_note_required` in `crates/mindtape-cli/src/flip.rs` with the evaluator the core task exposes, so a flip refuses the same note `mt check` would, before writing. Red test in `flip_tests.rs`: `mt flip done -m done` is refused under a `min-words = 10` policy. Then regenerate `schema/config.schema.json`, document the table form beside `note = "required"` in `www/content/config.typ` and `www/content/cli.typ`, and add a CHANGELOG.md entry.

Dogfood: give this repo's `done` status a note policy in `.mindtape/config.toml`.
