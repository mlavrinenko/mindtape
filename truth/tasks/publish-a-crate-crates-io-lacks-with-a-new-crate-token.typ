#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Publish a crate crates.io lacks with a new-crate token",
  status: done(2026, 10, 28),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

Trusted Publishing can't create a crate, so mindtape-cli, never published, needs one publish with a token scoped to publish-new before crates.io accepts a trusted publisher for it
