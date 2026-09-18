#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Speed up backlog search",
  status: cancelled(2026, 9, 5)[same problem as search-too-slow],
  priority: 7,
  tags: ("perf",),
  // docs:slice duplicates
  links: duplicates("search-too-slow.typ"),
  // docs:end
)

= Summary

Filed from a support thread before anyone checked the backlog.
