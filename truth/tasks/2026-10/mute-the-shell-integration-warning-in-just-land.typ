#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Mute the shell integration warning in just land",
  status: done(2026, 10, 14),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

A recipe cannot move its caller's shell, so the cd warning from `wt` on every land is noise
