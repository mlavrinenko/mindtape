#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Fix the release dry run's publish check",
  status: done(2026, 11, 3),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

A dry run leaves the manifests unbumped, so cargo verifies the new CLI against the old published crate and fails.
