#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices
#import "../utils/config-ref.typ": config-keys-table
#import "../utils/release.typ": cli-crate
#import "../generated/just.typ" as just

#let meta = (
  route: "/recurring/",
  label: "Recurring tasks",
  title: "Recurring tasks",
  summary: "How a task repeats with `every()` or `rrule()`, and how `mt recur` writes the next occurrence.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The blocks are slices of examples/recurring/, which `just check-rrule`
// validates with the rrule feature built in. The transcript is a run against
// a copy of it, made by www/generate-model.sh.
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

For a chore that comes back, like a weekly review or a yearly renewal, set
`recur:` on the task. `mt` never advances a series by itself, and no read
command changes a file. When you finish an occurrence, run `mt recur <task>`
to write the next one.

= Periods

`every()` takes exactly one of `days`, `weeks`, `months` or `years`:

#raw(
  render-slices(slices, (
    ("every-days", "every day"),
    ("every-weeks", "every Monday"),
    ("every-years", "every year, counted from the due date"),
  )),
  lang: "typ",
  block: true,
)

`on:` takes a set of weekdays, written with the prelude's names `mon` to
`sun`, and needs a `weeks` period. Each occurrence falls on the next listed
weekday in an on-week.

Which weeks are on-weeks? Weeks run Monday to Sunday and are counted from
Monday 2001-01-01, the same epoch for every series. With `weeks: N`, every Nth
week from it is an on-week. So `every(weeks: 2, on: (mon, thu))` falls on
Monday and Thursday of every second week. If you finish late in an off week,
the next occurrence still lands on the following on-week.

To pick which weeks, pass `start:` with a date. The week holding `start` is an
on-week, and so is every Nth week after it. So
`every(weeks: 2, on: (mon, thu), start: on(2026, 6, 1))` runs the weeks of
June 1 and June 15, and a series with `start: on(2026, 6, 8)` runs the other
weeks. A series without `start` keeps the 2001-01-01 epoch. `start` needs a
`weeks` period with an `on:` set, and a spawned occurrence keeps it.

= Policy

Four more arguments control the series:

- `anchor` sets the date the next occurrence counts from. `"completion"`
  counts from the day you run `mt recur`. `"due"` counts from the task's
  `due` date and skips any occurrence already past, so finishing late doesn't
  shift the schedule.
- `until` is the last date an occurrence may fall on.
- `count` is the number of occurrences left. `mt recur` lowers it by one each
  time.
- `materialize` sets what `mt recur` writes. `"spawn"` writes a new task file
  for the next occurrence. `"mutate"` resets the task's own status and dates
  for it. Spawn keeps a file per occurrence as history. Mutate keeps one file,
  and git holds the history.

`[tasks.recur]` sets the `default-anchor` and `materialize` a task gets when it
leaves them out, and limits the values a task may use:

#config-keys-table(json("../generated/config-schema.json"), "tasks.recur")

#raw(
  render-slices(slices, (
    ("every-policy", "every three months, with each policy argument set"),
  )),
  lang: "typ",
  block: true,
)

= Writing the next occurrence

`mt recur <task>` writes the next occurrence. With `"spawn"`, it writes a new
file and leaves the task you ran it on alone:

#terminal("recurring-spawn")

The new task carries a `recurred-from` edge to the one before it, so the
series is a chain of files:

#raw(
  render-slices(slices, (
    ("recurred-from", "the next day's task points back at the one before"),
  )),
  lang: "typ",
  block: true,
)

`mt recur` refuses a task that already has a successor, and a series that has
reached its `until` date or a `count` of 0. `--dry-run` prints what it would
write. `--advance <N>` writes the next N occurrences at once. `--skip` moves
the series past the next occurrence without writing one. The
#link(u("/cli/recur/"))[`mt recur` page] lists every flag.

= Calendar rules with rrule

When `every()` can't say it, such as the first Monday of each month or every
weekday, `rrule(...)` takes an RFC 5545 rule:

#raw(
  render-slices(slices, (
    ("rrule", "the first Monday of every month"),
  )),
  lang: "typ",
  block: true,
)

The released builds leave out the `rrule` cargo feature, so you need to
install `mt` with it:

// docs:hand-typed the crate name comes from the release workflow, and the build does not run cargo install
#raw(
  "cargo install " + cli-crate + " --features rrule",
  lang: "sh",
  block: true,
)

Then set `[tasks.recur] rrule = true` in the project's config. Without that
key, `mt check` reports each `rrule(...)` as disabled. Without the feature, it
reports that this build of `mt` can't read the rule. With both, it checks the
rule and quotes one that doesn't parse. #just.check-rrule() runs that check on
`examples/recurring/`.

An `rrule(...)` series counts from the task's `due` date, because RFC 5545
can't count from a completion date. `mt check` refuses
`anchor: "completion"` on one. A task without a `due` date counts from
today.
