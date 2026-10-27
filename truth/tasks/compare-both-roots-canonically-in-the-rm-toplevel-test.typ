#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Compare both roots canonically in the rm toplevel test",
  status: done(2026, 10, 28),
  tags: ("tweak",),
  estimate: (src-files: 1),
)

`repo_with` resolves its root with std, which keeps `\\?\` on Windows, while `mindtape::canonical` drops it
