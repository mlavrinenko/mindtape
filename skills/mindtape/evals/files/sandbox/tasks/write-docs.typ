#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Write onboarding docs",
  status: todo(2026, 6, 22),
  priority: 3,
  links: depends-on("add-dark-mode.typ"),
  tags: ("docs",),
)

= Summary

Document the new settings screen, including the dark mode toggle.
