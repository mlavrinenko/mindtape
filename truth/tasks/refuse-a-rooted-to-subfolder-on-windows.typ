#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Refuse a rooted --to subfolder on Windows",
  status: done(2026, 10, 27),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

On Windows `\etc` and `C:etc` are not absolute, so `mt add --to` let them through and the join left the destination root
