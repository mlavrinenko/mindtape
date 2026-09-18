#import "../utils/page.typ": page
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices
#import "../generated/just.typ" as just

#let meta = (
  route: "/recurring/",
  label: "Recurring",
  title: "Recurring tasks",
  summary: "One task that repeats: every() for the common cadences, rrule() for calendar rules, and the command that advances the series.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

`recur:` turns a task into a series. Authoring one costs a single line, and
advancing one costs a single command: nothing in MindTape moves a task to its
next occurrence on its own, and reading the backlog never mutates it. See
#link(u("/cli/"))[`mt recur`].

The slices below come from `examples/recurring/`, the project
#just.check-rrule() validates with the rrule engine built in.

#let slices = checked-slices(
  json("../generated/recurring.json"),
  "recurring",
  (
    "every-days",
    "every-weeks",
    "every-policy",
    "every-years",
    "rrule",
    "recurred-from",
  ),
)

= Periods

`every()` takes exactly one of `days`, `weeks`, `months`, `years`. A weekday
set narrows a week-scale rule: `every(weeks: 1, on: (mon,))` fires on Mondays
only. The weekday names are prelude constants, not strings to retype.

#raw(
  render-slices(slices, (
    ("every-days", "every day"),
    ("every-weeks", "one weekday a week, from the prelude's weekday set"),
    ("every-years", "a calendar promise, counted from its own due date"),
  )),
  lang: "typ",
  block: true,
)

= The policy that rides along

Every rule carries the same policy: `anchor` picks the date a series counts
from (`completion` by default, `due` for a calendar promise), `until` and
`count` stop it, and `materialize` chooses what #link(u("/cli/"))[`mt recur`]
writes — a new file per occurrence (`spawn`, the default) or the task refreshed
in place (`mutate`). A key the task omits resolves from `[tasks.recur]`;
#link(u("/config/"))[Configuration] carries that floor, defaults and all.

#raw(
  render-slices(slices, (
    ("every-policy", "month-scale, with every policy key set explicitly"),
  )),
  lang: "typ",
  block: true,
)

= The rrule escape hatch

Calendar rules `every()` cannot express — the first Monday of the month, the
last day of it, every weekday — go through `rrule(...)`:

#raw(
  render-slices(slices, (
    ("rrule", "the first Monday of every month, RFC 5545"),
  )),
  lang: "typ",
  block: true,
)

Two gates open it: the `rrule` cargo feature and `[tasks.recur] rrule = true`.
`mt check` parses the raw string either way and quotes a malformed one in the
error. RFC 5545 has no completion-relative primitive, so an `rrule` task is
date-anchored: it implies `anchor: "due"`, refuses `anchor: "completion"`, and
seeds its first instance from the task's `due`.

= What a spawn leaves behind

The successor #link(u("/cli/"))[`mt recur`] writes carries a `recurred-from`
edge to the occurrence it came from, so a series is a chain of real files, not
a counter inside one. #link(u("/links/"))[Task links] owns the kind, along with
the other five.

#raw(
  render-slices(slices, (
    ("recurred-from", "the successor's edge back to yesterday's task"),
  )),
  lang: "typ",
  block: true,
)
