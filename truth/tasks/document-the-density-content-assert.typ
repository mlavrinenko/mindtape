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
  estimate: (files: 4),
  kind: "docs",
  assignee: "agent",
  status: proposed(2026, 9, 14),
  links: depends-on("add-a-density-content-assert.typ"),
)

== Summary

The density variant lands in `add-a-density-content-assert.typ` as code and tests only; a project reading the config page still sees five content assertions and the generated schema still rejects the new key in an editor.


== Scope

Regenerate `schema/config.schema.json` from `mt config schema`, add the density assert beside the other five in `www/content/config.typ` with its `pattern`, `max` and `per` keys and the `section` default, and add a CHANGELOG.md entry under Unreleased. No behaviour changes.

== Dogfood

Add a `density` rule to this repo's `[[tasks.body.content.rules]]` in `.mindtape/config.toml`.
