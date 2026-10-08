#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Run the commit-msg hook's mt from the mt-dev build",
  status: done(2026, 10, 8),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

Same cold target/debug build as the pre-commit hook had
