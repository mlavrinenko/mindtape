#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Gate the write_file import on Unix in the atomic-write tests",
  status: done(2026, 10, 17),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

Only a Unix test calls write_file, so just check-windows warned that the import is unused
