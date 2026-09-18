#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Post standup notes",
  status: done(2026, 9, 14)[posted in the team channel],
  recur: every(days: 1),
)

= Summary

What changed, what is blocked, and who needs a decision today.
