#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Publish to crates.io through Trusted Publishing",
  status: done(2026, 10, 28),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The stored `CARGO_REGISTRY_TOKEN` from 2026-03-07 had expired. v0.3.0's publish job then failed with 403, and a token that crates.io mints per job can't expire on the shelf
