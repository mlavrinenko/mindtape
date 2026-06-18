#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Status note authored as a trailing content block",
  status: done(2026, 6, 18)[prelude takes trailing note; harvest projects content to plain text],
  priority: "normal",
  estimate: (hours: 1),
  relates-to: (link("harvest-reads-standard-link.typ"),),
  tags: ("substrate",),
)

== Summary

A status note read as `done(y, m, d, note: "…")` — a named string argument. The
ergonomic Typst form is a trailing content block: `done(y, m, d)[note text]`.
Markup is what notes are, so author them as content, not a quoted string.

== Decision

The prelude status shortcuts take the optional note as a trailing positional
content block; `status()` keeps its `note:` slot for direct callers. Teach
`typst-harvest` to project a content value to its concatenated plain text so the
harvested note is a clean string. This stays consumer-blind: projecting content
to plain text (and a `link()` to its destination) is a generic Typst-reading
rule, nothing task-specific.

== Scope

- `lib/prelude.typ`: `_mk` accepts `..note`, threading `note.pos().at(0)` into
  `status`. The `done(y, m, d)[note]` call site works; bare `done(y, m, d)` keeps
  a `none` note.
- `crates/typst-harvest`: `convert` projects `Value::Content` via `content_str` —
  link URL destination first, else `Content::plain_text`. Other values keep the
  repr fallback.
- Regression test: a plain-text content value converts to its text.
- Rewrite `harvest-reads-standard-link.typ` to the trailing-block note form.
