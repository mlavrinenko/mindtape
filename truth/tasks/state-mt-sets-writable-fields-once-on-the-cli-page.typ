#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "State mt set's writable fields once on the CLI page",
  status: done(2026, 10, 15),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The page prose listed the fields mt set writes, which the generated long help already prints from the table set.rs reads
