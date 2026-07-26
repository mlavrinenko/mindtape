#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Fix login redirect bug",
  status: wip(2026, 6, 25)[reproduced on staging],
  priority: 8,
  tags: ("backend", "auth"),
)

= Summary

Users are redirected to a blank page after login when the session cookie is
already set. Clear the stale cookie before issuing the new one.
