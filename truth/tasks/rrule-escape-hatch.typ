#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "RRULE escape hatch for recurrence",
  priority: ice(
    impact: 4,
    confidence: 0.6,
    ease: 5,
  ),
  tags: (),
  links: depends-on("recurring-tasks.typ"),
  status: wip(2026, 7, 10),
)

== Summary

Add a raw `rrule("FREQ=…")` recurrence rule for the calendar-complex cadences
the typed `every()` cannot express — "first Monday of the month"
(`FREQ=MONTHLY;BYDAY=1MO`), "last day of month" (`BYMONTHDAY=-1`), "every
weekday" (`BYDAY=MO,TU,WE,TH,FR`). Expand it with the `rrule` crate
(github.com/fmeringdal/rust-rrule). Feature-gated so the common period path stays
dependency-free: `every()` covers the 90% with plain chrono arithmetic, and only
a project that actually writes an `rrule(...)` pulls in chrono-tz + regex.

== Decisions

- Behind a cargo feature (`rrule`), off by default; gated in config by
  `[tasks.recur] rrule`.
- `rrule(...)` is date-anchored only: it implies `anchor: "due"` and rejects
  `anchor: "completion"`. RFC 5545 has no completion-relative primitive — a
  completion anchor would mean re-seeding DTSTART every advance, at which point
  `every()` is the simpler tool. Validation enforces this.
- DTSTART is seeded from the task's `due`; the next occurrence is the rule's
  first instance strictly after it (`RRuleSet::after`-style). Same policy args as
  `every()` apply: `until`, `count`.
- Materialization (spawn vs mutate) and linking are unchanged — this task only
  adds a second rule engine behind the existing `recur` field.

== Constraints

- Bound generation with the crate's iteration limit to avoid a pathological rule
  looping; surface a clear error rather than hang.
- `mt check` must validate an `rrule` string at parse time and reject a malformed
  one with a precise message (quote the offending rule).
- Keep the non-`rrule` build free of chrono-tz / regex (verify via a
  no-default-features build in CI).

== Scope

- prelude `rrule(rule, until:, count:)` helper (shares the `recur` dict shape).
- cargo feature `rrule`; expander over the `rrule` crate.
- validation: feature/config gate, `anchor: "completion"` rejection, malformed
  rule rejection.
- Tests: first Monday, last day of month, every weekday; `until` / `count` end;
  the completion-anchor rejection; a no-default-features build compiles.
