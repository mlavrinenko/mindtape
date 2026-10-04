#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Fix the refund rounding",
  status: todo(2026, 9, 29),
  kind: "fix",
  priority: 5,
  tags: ("billing",),
  x-team: "payments",
)

= Summary

A refund of 10.005 rounds down, so the customer gets a cent less.
