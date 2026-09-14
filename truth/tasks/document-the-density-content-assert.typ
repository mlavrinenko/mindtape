#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Document the density content assert",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("mt", "docs"),
  estimate: (files: 3),
  kind: "docs",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("add-a-density-content-assert.typ"),
)

== Summary

The density variant landed in `add-a-density-content-assert.typ` as code, tests and the regenerated `schema/config.schema.json`. A project reading the config page still sees five content assertions, and this repo polices no prose density at all.

== Scope

Add the density assert beside the other five in `www/content/config.typ`: its `pattern`, `max` and `per` keys, the `section` default, and the unit-split rules documented at `density_units` in `crates/mindtape/src/content_gate.rs` (fenced code dropped, blank-line paragraphs, each list item its own paragraph, sentences ending at `.`, `!` or `?`). Add a CHANGELOG.md entry under Unreleased. No behaviour changes.

Land this rule in `.mindtape/config.toml`, after the other `[[tasks.body.content.rules]]` entries:

```toml
[[tasks.body.content.rules]]
path = [{ match = ".*" }]
assert = { density = { pattern = "—", max = 1, per = "paragraph" } }
when = "!terminal"
message = "more than one em-dash in a paragraph reads as generated prose; use a comma, a colon or a second sentence"
```

One per paragraph is the threshold the Vale packages behind the assert measured, and `!terminal` leaves closed history alone. Measured on 2026-09-14: of 68 open-task paragraphs, 63 carried no em-dash, 2 carried one and 3 carried two, and those 3 were rewritten under the limit the same day. Run `mt check` before committing; if a task filed since trips the rule, rewrite that paragraph in a separate `task:` commit rather than raising `max`.

== Dogfood

Built in: the rule lands in this repo's `.mindtape/config.toml`.
