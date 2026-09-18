#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Keep the legacy CLI working",
  status: cancelled(2026, 8, 22)[the rewrite lands instead],
  priority: 3,
  tags: ("cli",),
)

= Summary

Freeze the old CLI where it is: bug fixes only, no new flags.
