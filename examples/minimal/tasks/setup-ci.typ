#import "@local/mindtape:0.2.0": *
#show: task.with(
  title: "Set up CI pipeline",
  status: done(2026, 6, 10)[merged in #42],
  priority: 5,
  tags: ("tooling",),
)

= Summary

GitHub Actions workflow running fmt, clippy, and tests on every push.
