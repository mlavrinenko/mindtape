#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Run the git hooks' mt from the mt-dev build",
  status: done(2026, 10, 8),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The hooks built mt into target/debug, which nothing else warms, so a commit in a fresh worktree compiled cold for up to 15 minutes
