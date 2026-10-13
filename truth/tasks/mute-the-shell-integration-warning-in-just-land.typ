#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Mute the shell integration warning in just land",
  status: done(2026, 10, 14),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

A recipe cannot move its caller's shell, so wt's cd warning on every land is noise
