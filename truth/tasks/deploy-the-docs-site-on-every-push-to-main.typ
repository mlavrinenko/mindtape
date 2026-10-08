#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Deploy the docs site on every push to main",
  status: done(2026, 10, 8),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The site renders captures of mt built from crates/ and lib/, so a www/ path filter left it stale after a CLI-only change
