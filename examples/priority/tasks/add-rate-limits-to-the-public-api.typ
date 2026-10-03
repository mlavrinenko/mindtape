#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Add rate limits to the public API",
  status: todo(2026, 9, 28),
  // docs:slice ice-call
  priority: ice(impact: 10, confidence: 0.9, ease: 9),
  // docs:end
)

= Summary

One client can take the API down for everyone.
