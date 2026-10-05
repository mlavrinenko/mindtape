#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Return the unreleased 0.3.0 changelog entries to Unreleased",
  status: done(2026, 10, 5),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

No v0.3.0 tag or crates.io release exists, so the dated 0.3.0 section claimed a release that never happened
