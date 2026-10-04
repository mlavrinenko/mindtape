#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rotate the JWT signing keys",
  status: wip(2026, 9, 28),
  kind: "chore",
  priority: 7,
  tags: ("auth", "security"),
  x-team: "platform",
)

= Summary

The signing key has not changed since launch.
