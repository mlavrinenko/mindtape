#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Retry failed invoice payments",
  status: todo(2026, 9, 25),
  kind: "feat",
  priority: 6,
  tags: ("billing",),
  x-team: "payments",
  links: depends-on("rotate-the-signing-keys.typ"),
)

= Summary

The payment webhook signs its callbacks with the key the rotation replaces.
