#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rotate the signing key",
  status: proposed(2026, 9, 25),
  recur: every(months: 3),
  tags: ("security",),
  x-ops: (runbook: "docs/runbooks/rotate-key.md"),
)
