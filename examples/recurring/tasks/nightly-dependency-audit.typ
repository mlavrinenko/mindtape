#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Audit dependencies nightly",
  status: todo(2026, 9, 14),
  priority: 6,
  tags: ("deps",),
  // docs:slice every-days
  recur: every(days: 1),
  // docs:end
)

= Summary

Run the advisory audit after the nightly build and file anything exploitable.
