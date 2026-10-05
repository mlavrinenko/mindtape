#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Let every() take a start date that fixes its week parity",
  kind: "feat",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt",),
  estimate: (src-files: 5),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a project picks its own alternate weeks without a config key],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 5),
)

== Problem

+ Run `mt recur` on a task declaring `every(weeks: 2, on: (mon, thu))` whose last `due` is Thursday 2026-06-04.
+ 🔴 The next date follows `SERIES_EPOCH`, the Monday 2001-01-01 in `crates/mindtape/src/recur_dates.rs`, which counts every series' week parity. No argument lets a project pick which alternate weeks the series runs on.


== TO-BE

- 🟢 `every(weeks: 2, on: (mon, thu), start: on(2026, 6, 1))` counts parity from the Monday of the week holding `start`: that week and every second one after it are on-weeks.
- 🟢 Without `start` the global epoch stays the default, so no existing series moves.
- 🟢 `start` rides in the `rule` dict of `recur`, through the canonical JSON and `emit`, so a spawned instance keeps it.
- 🟢 `www/content/recurring.typ` and `www/content/dsl.typ` document the argument.


== Changes

- `crates/mindtape/src/recur_dates_tests.rs`: the red test first, a `start` fixing the on-weeks and the epoch staying the default without it.
- `crates/mindtape/src/recur_dates.rs`: `is_on_week` counts from the rule's `start` Monday, else `SERIES_EPOCH`.
- `crates/mindtape/src/recur.rs`, `crates/mindtape/src/recur/canonical.rs`, `crates/mindtape/src/emit.rs`: `Rule::Period` carries `start`; parse it, round-trip it, emit it.
- `lib/prelude.typ`: `every()` takes `start:` and puts it in the `rule` dict; `_fmt-recur-rule` shows it.
- `www/content/recurring.typ`, `www/content/dsl.typ`: document `start`.


== Alternatives

- A `[tasks.recur] epoch` config key: one epoch per project, so two series in one project cannot run on opposite weeks.
- Derive parity from the task's first `due`: it silently moves when someone edits `due`, and a spawned instance has a new `due`.
- Put `start` at the top level of the recur dict beside `until`: it only means something for a weekday period, so it belongs to the rule.


== Out of scope

- `rrule(...)`, whose `DTSTART` already fixes its own phase.
- Month and year periods, which have no parity.


== Dogfood

Not applicable, this repo has no recurring task.
