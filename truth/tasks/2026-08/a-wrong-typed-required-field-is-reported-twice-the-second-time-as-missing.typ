#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "A wrong-typed required field is reported twice, the second time as missing",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("substrate",),
  estimate: (files: 2),
  assignee: "agent",
  status: done(
    2026,
    8,
    21,
  )[a field-level shape error masks the required answer at each check's input. An element error still lets it fire, since the field itself read],
  links: related(
    "report-a-wrong-typed-element-inside-a-well-shaped-core-field.typ",
  )[
    the shape-error channel this is the loud half of, found while closing it
  ],
)

== Summary

`Task::from_marker` reports a wrong-typed core field, and the field still
degrades to absent, so a `required` field reports twice, the second time
wrongly. `status: "wip"` yields both:

```text
status: expected a status like `wip(2026, 6, 14)`, found a string
status: missing required status
```

The second line is false: the author wrote a status. It sends them to add a
field that is already there, the exact misdirection the shape-error work set out
to end. The work closed that for the silent half and left it open for the loud
one. Every other `required` field a shape error empties (`priority`, `estimate`,
`assignee`, `kind`) reads the same way.

== Scope

- suppress the `required` violation for a field already carrying a field-level
  shape error: `status`, not `status.date`. An element error leaves the field
  readable, and the required check on it is then correct
- decide where that lives: `check_required` reading the shape-error set, or
  `validate::check_all` filtering after the fact
- regression test: a task whose only mistake is a wrong-typed required field
  reports exactly one violation, and that violation reports the type
