#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Honour the week count in every(weeks, on)",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  estimate: (src-files: 2),
  tags: ("found", "mt"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the recurring page can state one rule for `on:`],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 3),
)

== Problem

+ Verifying the /recurring/ page's weekday claims against recur\_dates.rs.
+ 🔴 #raw("crates/mindtape/src/recur_dates.rs"): With an \`on:\` weekday set, \`mt recur\` steps to the next listed weekday and ignores the week count, so \`every(weeks: 2, on: (mon,))\` repeats every Monday.

```rust
Rule::Period { weekdays, .. } if !weekdays.is_empty() => next_weekday_after(from, weekdays),
```
`next_weekday_after` scans the 7 days after `from` and never reads `value`.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/recur_dates.rs").

== TO-BE

- 🟢 `every(weeks: 2, on: (mon, thu))` falls on Monday and Thursday of every second week.
- 🟢 `/recurring/` states that rule, and drops its sentence that the number of weeks has no effect.


== Changes

- `crates/mindtape/src/recur_dates.rs`: after the last listed weekday of a week, skip `value - 1` weeks.
- `www/content/recurring.typ`: describe the week count with `on:`.


== Rollback

- Revert the commit.


== Out of scope

- `rrule(...)`, which has its own engine.


== Dogfood

- Not applicable: this repo has no weekday series.

