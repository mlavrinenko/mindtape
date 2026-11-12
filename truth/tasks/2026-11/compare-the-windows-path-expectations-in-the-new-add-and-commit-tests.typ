#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Compare the Windows path expectations in the new add and commit tests",
  status: done(2026, 11, 12),
  tags: ("tweak",),
)

The new tests compared native Windows paths with `/`, so the release commit failed the Windows job
