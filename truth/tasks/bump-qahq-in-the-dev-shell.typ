#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Bump qahq in the dev shell",
  status: done(2026, 10, 31),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

qahq's lock now shares naersk and flake-utils, cutting the dev lock from 167 to 147 nodes
