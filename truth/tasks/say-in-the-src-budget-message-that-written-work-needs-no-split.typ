#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Say in the src-budget message that written work needs no split",
  status: done(2026, 10, 11),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The old message read as a request to split even when no work was left, so an agent asked before closing
