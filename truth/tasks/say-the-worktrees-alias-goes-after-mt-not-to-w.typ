#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Say the worktrees alias goes after mt, not to -w",
  status: done(2026, 10, 8),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

mt worktrees help said the \@name it lists is passed to -w, but mt -w \@main ls fails while mt \@main ls works
