#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Keep every(weeks: N, on: …) on its weeks after a late finish",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 6,
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
      docs-match-code: helps[/recurring/ can promise every Nth week and keep it],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 4),
)

== Problem

+ Reviewing commit f8cc8400 with a pinned date in a scratch project
+ 🔴 #raw("crates/mindtape/src/recur_dates.rs"): next\_weekday\_after treats the week of its start date as an on-week, so finishing late into an off week shifts the whole series by a week.

`every(weeks: 2, on: (mon, thu))`, due Thursday 2026-06-04, finished Monday 2026-06-08:
```
$ MINDTAPE_NOW=2026-06-08 mt recur sync --dry-run --advance 4
would spawn /tmp/rvw-mt-94/tasks/hold-the-sync-2026-06-11.typ (due 2026-06-11)
would spawn /tmp/rvw-mt-94/tasks/hold-the-sync-2026-06-22.typ (due 2026-06-22)
```


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("crates/mindtape/src/recur_dates.rs").

== TO-BE

- 🟢 Week parity comes from a fixed series epoch, not the completion week, so the series above falls on 2026-06-15 next.
- 🟢 `/recurring/` names the epoch.


== Changes

- `crates/mindtape/src/recur_dates.rs`: pick the on-weeks by weeks since the series epoch; a red test starts in an off week.
- `www/content/recurring.typ`: say which date fixes the weeks.


== Rollback

- Revert the commit; the series counts weeks from each completion again.


== Out of scope

- `rrule(...)`, which has its own engine.
- `every()` without `on:`.


== Dogfood

Not applicable: this repo has no weekday series.
