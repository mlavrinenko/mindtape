#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Print a short diff when just fmt-check fails on a justfile",
  status: done(2026, 10, 13),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

`just --fmt --check` printed about 900 lines of context to report one misplaced blank line.
