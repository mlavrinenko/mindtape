#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Tell contributors to fix a bug rather than document it",
  status: done(2026, 10, 18),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The owner wants the docs to describe working behavior only, so a bug found while writing gets fixed, never noted.
