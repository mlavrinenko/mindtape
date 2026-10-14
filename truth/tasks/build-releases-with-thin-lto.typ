#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Build releases with thin LTO",
  status: done(2026, 10, 14),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

A one-line change rebuilt the release `mt` in 271 s with fat LTO and takes 102 s with thin, for a binary 5% larger.
