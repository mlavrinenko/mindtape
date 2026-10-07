#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Call outdatty check directly in the drift gate",
  status: done(2026, 10, 7),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

outdatty 0.7.0 lists only the failing groups, so the recipe needs no jq
