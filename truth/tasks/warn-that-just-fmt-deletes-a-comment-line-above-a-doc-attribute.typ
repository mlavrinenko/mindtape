#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Warn that just --fmt deletes a comment line above a doc attribute",
  status: done(2026, 10, 7),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

just 1.46 dropped a line of the outdatty-check comment on format
