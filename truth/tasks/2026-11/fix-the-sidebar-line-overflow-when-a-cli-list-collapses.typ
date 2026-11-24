#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Fix the sidebar line overflow when a CLI list collapses",
  status: done(2026, 11, 24),
  tags: ("tweak",),
  assignee: "agent",
)

A closed `<details>` keeps the bounding boxes of its hidden rows, so the green segment still ended at `mt flip` and ran past the collapsed group.
