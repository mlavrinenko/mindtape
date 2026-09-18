#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Drop the legacy CLI",
  status: todo(2026, 9, 2),
  priority: 5,
  tags: ("cli",),
  // docs:slice supersedes-and-edge
  links: (
    supersedes("legacy-cli.typ"),
    edge("discharged-by", "https://github.com/acme/api/pull/42"),
  ),
  // docs:end
)

= Summary

Delete the old CLI once the rewrite covers its last flag, and keep the PR that
did it on the task.
