#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Plain, copy-friendly list output",
  status: done(2026, 6, 19),
  priority: 5,
  estimate: (hours: 4),
  depends-on: (link("tty-render-discipline.typ"),),
  relates-to: (link("field-display.typ"), link("list-handle-ordinal-check.typ")),
  tags: ("cli",),
)

== Summary

Drop the comfy-table `UTF8_FULL` box-drawing table: it is hard to copy, grep, and
diff, and carries per-row and per-table overhead. Render space-padded, borderless
columns instead — one line per task, one header line. Truncate the title to a
single line on a TTY (unicode-width aware, ellipsis) and leave it full when piped.
json and csv stay lossless; add a tab-separated plain mode for pipes.

== Scope

- Borderless, padded renderer replacing comfy-table for the human format.
- Title truncation to terminal width on a TTY; no truncation off a TTY.
- Tab-separated plain output for pipes; json / csv unchanged and lossless.
- Column set and order revisited (handle, status, priority, due, title). Colour
  and glyph display land with the field legend, not here.
