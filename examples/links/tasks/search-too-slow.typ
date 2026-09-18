#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Search is too slow on large backlogs",
  status: todo(2026, 9, 4),
  priority: 7,
  tags: ("perf",),
)

= Summary

P95 query time crosses 800 ms once a project passes fifty thousand rows.
