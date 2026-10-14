#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Link dev builds with mold",
  status: done(2026, 10, 14),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

An edit to `mindtape` relinked the test binaries in 10.2 s with ld and takes 4.6 s with mold, which the dev shell already shipped.
