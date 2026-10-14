#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Let the release check job read CI's caches",
  status: done(2026, 10, 14),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

The release `check` job reruns `just check` on the tag with no cache, so it compiled the workspace cold on every release.
