#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Say where the Typst dependencies' source and HVal live",
  status: done(2026, 10, 4),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

Agents searched the whole filesystem for HVal, which lives in typst-world, not this repo
