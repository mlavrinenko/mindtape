#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Build the merge-driver and shell test helpers on Unix only",
  status: done(2026, 10, 17),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

Only Unix e2e modules call them, so just check-windows warned that their re-exports are unused
