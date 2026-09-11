#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Run the daily Hanon warm-up",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 8,
  ),
  estimate: (minutes: 15),
  tags: ("warmup", "daily"),
  due: on(2026, 7, 11),
  recur: every(days: 1),
  status: wip(2026, 7, 9)[folded into the daily routine, exercises 1-5],
)

== Notes

Exercises 1-5 before anything else: evens out finger independence and stops
the rest of the session from reinforcing sloppy technique. Advances daily via
`mt recur` on completion — a fresh occurrence each day, never a background
job.
