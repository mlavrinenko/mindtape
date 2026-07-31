#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Report a wrong-typed core field instead of coercing it to empty",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("substrate",),
  estimate: (files: 3),
  status: proposed(2026, 7, 31),
  links: related("link-target-in-edge-helper-yields-zero-edges.typ")[
    the other half of the same layer split
  ],
)

== Summary

A field that reaches the marker with the wrong type is silently coerced to
empty by `Task::from_marker`, so the half of the authoring surface the prelude
cannot police stays unpoliced.

Verified 2026-07-31: `tags: ("backend")` — a Typst one-element tuple missing
its trailing comma, so a plain string, not an array — loads as no tags at all;
`links: "a.typ"` loads as no edges; `estimate: "2 days"` loads as an empty unit
dict. Unlike the `link()` trap these values DO reach the marker, so the Rust
layer can see and report them. It does not: `from_marker` has no error channel,
and `Task` keeps no raw marker to check later.

== Scope

- give the marker projection an error channel, and decide where a shape
  violation surfaces — a load error (the file fails, as a prelude assert does)
  or an `mt check` violation (the task loads and reports)
- cover `tags`, `links`, `estimate`, and the `on:` weekday set of `recur`
- out of scope: asserting these in the prelude. A `text` slot's default is a
  string by construction (`estimate: slot("estimate", kind: "text", default:
  "(files: 2)")`), so a Typst-side type assert would break template
  previewability
- out of scope: the argument shapes the prelude already rejects, which never
  reach the marker at all
