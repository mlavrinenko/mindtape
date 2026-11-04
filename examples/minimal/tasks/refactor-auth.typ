#import "@local/mindtape:0.4.0": *
#show: task.with(
  title: "Refactor auth middleware",
  status: todo(2026, 6, 18),
  priority: 6,
  tags: ("backend", "auth"),
)

= Summary

Split the monolithic auth middleware into composable guards.
