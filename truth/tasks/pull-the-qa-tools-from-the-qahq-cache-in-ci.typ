#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Pull the QA tools from the qahq cache in CI",
  status: done(2026, 10, 30),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

CI read only the mlavrinenko cache, so it built qahq's cargo-crap 0.3.1 itself and pushed a second copy of it to the free 5 GB cache.
