#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal
#import "../utils/site.typ": u
#import "../utils/problems.typ": own-config, problem-config

#let meta = (
  route: "/one-task-at-a-time/",
  label: "One task at a time",
  title: "Work started before the last task is done",
  summary: "Allow one task in progress, and refuse to start a task before the tasks it depends on are done.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

An agent starts three tasks and finishes none. The working tree holds parts of
all three, and no commit can name just one. Or it starts a task whose
dependency is still open, and writes code against a client that doesn't exist
yet.

= The config

#problem-config("one-task-at-a-time")

= What `mt` does

One task is already in progress, so `mt wip` won't start a second. It also
won't start a task that waits on an open dependency, and `mt ls blocked` tells
you which one:

#terminal("one-task-at-a-time")

= How it works

The `one-wip` rule counts tasks in `wip` with `count()` and fails if there's
more than one. Every command that writes a task checks the rules first, so
`mt wip` won't start a second one. #link(u("/rules/"))[Rules] covers
work-in-progress caps, and #link(u("/filters/"))[Filter language] covers
`count()`.

`tasks.status` lists the statuses a task may hold. `terminal = true` marks
`done` as the status that ends a task. `unblocked = true` on `wip` makes
`mt wip` refuse a task while a `depends-on` edge points at a task that isn't
done. `--force` overrides it. `mt ls blocked` and its `why` column list each
waiting task and what it waits on. #link(u("/statuses/"))[Statuses] covers
terminal and unblocked statuses, and #link(u("/links/"))[Links] covers the
edge kinds.

This repo caps its own work in progress the same way in #own-config.
