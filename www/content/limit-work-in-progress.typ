#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/recipes.typ": own-config, recipe-config

#let meta = (
  route: "/limit-work-in-progress/",
  label: "Limit work in progress",
  title: "Limit work in progress",
  summary: "Allow one milestone and one task in progress, and refuse to start a task before the tasks it depends on are done.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

An agent starts three tasks and finishes none. The working tree holds parts of
all three, and no commit can name just one. Or it starts a task whose
dependency is still open, and writes code against a client that doesn't exist
yet.

= Config

#recipe-config("limit-work-in-progress")

= Behavior

The milestone "Harden session tokens" and one task under it are in progress,
so `mt wip` won't start a second task. It also won't start a task that waits
on an open dependency, and `mt ls blocked` tells you which one:

#terminal("limit-work-in-progress")

= Deep dive

A milestone groups tasks and stays open until they're done, often across
several sessions, so it gets a cap of its own. The `one-milestone` rule uses
`count()` to count the milestones in `wip`, and fails when there's more than
one. `one-task` does the same for every other task, one with no `kind`
included, because a missing field equals no value. Every command that writes
a task checks the rules first. #link(u("/rules/"))[Rules] covers
work-in-progress caps, and #link(u("/filters/"))[Filter language] covers
`count()`.

A task names its milestone with a `parent` link, and `mt ls` counts a
milestone's tasks in its `children` and `progress` columns.
#link(u("/links/"))[Task links] covers the edge kinds.

`tasks.status` lists the statuses a task may hold. `terminal = true` marks
`done` as the status that ends a task. `unblocked = true` on `wip` makes
`mt wip` refuse a task while a `depends-on` edge points at a task that isn't
done. `--force` overrides it. `mt ls blocked` and its `why` column list each
waiting task and what it waits on. #link(u("/statuses/"))[Statuses] covers
terminal and unblocked statuses.

This repo caps its own work in progress the same way in #own-config.
