#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Report a wrong-typed element inside a well-shaped core field",
  priority: ice(
    impact: 6,
    confidence: 0.85,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("substrate",),
  estimate: (files: 3),
  assignee: "agent",
  status: done(
    2026,
    8,
    21,
  )[core fields report their elements, tags and recur.on included. The card render is total, so tags reports a real shape error],
  links: related(
    "../2026-07/report-a-wrong-typed-core-field-instead-of-coercing-it-to-empty.typ",
  )[
    built the shape-error channel this extends from the field down to its
    elements.
  ]
    + related("array-form-links-silently-dropped.typ")[
      the one instance of this pattern already closed, and the audit it asked
      for is what filed this.
    ]
    + related(
      "a-wrong-typed-required-field-is-reported-twice-the-second-time-as-missing.typ",
    )[
      the loud half this leaves open: a field that reports an error still
      degrades to absent, so a required one also reports as missing on top.
    ],
)

== Summary

`Task::from_marker` reports a wrong-typed core field, but only at _field_ level:
`links: "a.typ"` and `estimate: "2 days"` are caught. A wrong-typed _element_
inside a well-shaped field still disappears in silence. Element errors carry a
position, as in `tags[1]`, `estimate.days`, `priority.inputs.ease`, `status.date`
and `recur.on[1]`.

== Verified

Confirmed 2026-08-20 against the debug binary, one probe task per row. Each
loads clean, and `mt check` stays green:

#table(
  columns: 2,
  table.header([authored], [loads as]),
  [`estimate: (days: "two")`], [`{}`],
  [`priority: (8, 9)`], [`null`],
  [`kind: 7`], [`null`],
  [`assignee: 5`], [`null`],
  [`due: 42`], [`null`],
)

Caught only by accident: `tags: ("x", 7)` and `tags: ("x", ("y", "z"))` fail at
_eval_, because the prelude's `_tag-chip` calls `raw(t)` on every entry, not
because anything validates them. The message omits both the field and the task,
and it would disappear the moment the render stopped calling `raw`.

The same pattern repeats in every `filter_map` and `?` in
`crates/mindtape/src/model.rs` that discards a value not matching its expected
type instead of reporting it. `parse_num_map` drops a non-numeric unit,
`parse_str_array` a non-string tag, `parse_num_value` and `parse_status` return
`none` for anything they cannot read, and `field_str` does the same for every
scalar slot.

== Scope

- extend the shape-error channel from the field to its elements, across every
  core field `from_marker` projects
- a report identifies the field and the offending position, the way
  `crate::link::collect_links` does for `links[1][0]`
- decide whether the accidental eval-time catch on `tags` stays as it is or
  becomes a real shape error like the rest. It is the only field whose element
  check lives in the render layer, which is not a layer that owes anyone
  validation
- out of scope: `links`, closed by `array-form-links-silently-dropped.typ`
- out of scope: the argument shapes the prelude rejects before the marker, which
  never reach `from_marker` at all
