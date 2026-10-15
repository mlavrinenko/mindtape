#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Let a commit of only task files skip the gate freshness check",
  status: done(2026, 10, 15),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

A fresh fork's gates never ran, so filing its task first needed a five-minute just check
