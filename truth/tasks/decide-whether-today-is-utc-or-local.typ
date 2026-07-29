#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Decide whether today is UTC or local",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 3),
  status: done(2026, 7, 29),
)

== Summary

mindtape::today\_ymd() derives today from SystemTime::now() epoch seconds, so every dated write stamps UTC. At UTC+3 anything filed after 21:00 local carries yesterday date: a task filed at 01:00 MSK on the 29th was stamped 2026-07-28. The choice is defensible, no timezone dependency and one reading per process, but it is undocumented and surprising, and it also skews the date filters (now, stale, due) by up to a day for anyone east of UTC.

== Scope

- Decide: keep UTC and document it, or resolve the local offset.
- If local: pick the mechanism. The time crate local offset is unsound in a multithreaded process, chrono Local or jiff are the realistic options, and MINDTAPE\_NOW must keep overriding whatever lands.
- Whichever way it goes, say so where a user meets it (docs and the today related filters), and add a test that pins the answer.

== Decision

Local. A task filed at 01:00 belongs to that day, and mt is a single-user tool run on the machine whose calendar the user is looking at; a stamp that disagrees with the wall clock is a bug report waiting to happen, and the skew hits now, stale and the overdue marker too, not only the writes.

Mechanism: chrono Local, via the clock feature on a dependency the workspace already carries (both crates pin chrono for NaiveDate arithmetic), rather than a second date crate. jiff would be the pick on a green field, but it buys nothing here beyond a duplicate tzdb. Chrono resolves the zone itself since 0.4.20 (its own tzfile parser plus iana-time-zone, no localtime\_r), so the multithreaded unsoundness that rules out time does not apply. Cost is iana-time-zone; the zone lookup is cached per thread.

Shape:

- date.rs keeps one clock read (`today_epoch_days`), and only `local_epoch_days` knows about zones. Downstream everything is a plain YYYY-MM-DD civil date with no offset attached — dates in task files are civil dates, compared against whatever today is where the command runs.
- MINDTAPE\_NOW still wins, and its value is taken verbatim: a trailing RFC 3339 offset shifts nothing, because the pinned value is the date to stamp.
- No config knob. A per-project timezone would make the same file mean different things to different clones for no gain; TZ already covers the override case on Unix.

Pinned by: `local_epoch_days_reads_the_offsets_own_calendar_date` (one instant, +03:00 and -05:00, two different days) in date.rs, and `today_is_the_local_calendar_date_not_utc` in date\_pin\_e2e.rs, which files a task under UTC+14 and UTC-12 and asserts the stamps differ — 26 hours apart, so it never depends on when the suite runs. The e2e half is cfg(unix): chrono reads the Windows zone through the platform API, which no env var overrides.

Documented on the config page (Environment variables), the agents page and the filters page.
