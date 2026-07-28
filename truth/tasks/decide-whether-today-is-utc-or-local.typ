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
  status: proposed(2026, 7, 28),
)

== Summary

mindtape::today\_ymd() derives today from SystemTime::now() epoch seconds, so every dated write stamps UTC. At UTC+3 anything filed after 21:00 local carries yesterday date: a task filed at 01:00 MSK on the 29th was stamped 2026-07-28. The choice is defensible, no timezone dependency and one reading per process, but it is undocumented and surprising, and it also skews the date filters (now, stale, due) by up to a day for anyone east of UTC.

== Scope

- Decide: keep UTC and document it, or resolve the local offset.
- If local: pick the mechanism. The time crate local offset is unsound in a multithreaded process, chrono Local or jiff are the realistic options, and MINDTAPE\_NOW must keep overriding whatever lands.
- Whichever way it goes, say so where a user meets it (docs and the today related filters), and add a test that pins the answer.
