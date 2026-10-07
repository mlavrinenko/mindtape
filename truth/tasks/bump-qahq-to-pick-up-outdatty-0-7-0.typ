#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Bump qahq to pick up outdatty 0.7.0",
  status: done(2026, 10, 7),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

outdatty 0.7.0 prints only the failing groups in a plain check
