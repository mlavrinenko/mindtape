#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Update the dev shell's mmz to 0.12.1",
  status: done(2026, 10, 16),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

mmz 0.12.1 prints one line from mmz --import, so just land stops listing every gate record
