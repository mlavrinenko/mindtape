#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Groom the backlog",
  status: todo(2026, 9, 14),
  priority: 4,
  tags: ("process",),
  // docs:slice every-weeks
  recur: every(weeks: 1, on: (mon,)),
  // docs:end
)

= Summary

Close what went stale, re-rank the top of the list, and pick the week's bet.
