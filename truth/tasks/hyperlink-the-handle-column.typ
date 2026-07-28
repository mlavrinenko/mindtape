#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Hyperlink the handle column",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 4),
  links: depends-on("hyperlink-the-file-column-with-osc-8.typ"),
  status: proposed(2026, 7, 28),
)

== Summary

Once OSC 8 wrapping exists, the `#` handle is the natural thing to click. It is
the leftmost cell, it is already what you read to identify a row, and unlike
the `file` column it costs no horizontal space — the link rides on characters
that are on screen regardless. A listing could then be click-navigable without
showing a path column at all.

== Decisions

- Reuses the `[ls] hyperlinks` switch rather than introducing a second opt-in
  surface. One question ("do links work in your terminal") should have one
  answer.
- The target is the task's own file, built through the same
  `[ls] hyperlink-target` template.

== Open questions

- Whether per-column control is wanted (`[ls.columns.<name>] link = true`) or
  whether a fixed set of linked columns is enough. Two linked columns is the
  point at which this stops being obvious.
- Whether linking `#` makes the `file` column redundant for the click use case,
  and if so whether `file` should ever appear in a default column set.
