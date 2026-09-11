#import "../utils/page.typ": page
#import "../utils/ui.typ": callout

#let meta = (
  route: "/estimates/",
  label: "Estimates",
  title: "Estimate bounds and working-tree drift",
  summary: "How tasks.estimate bounds a task's declared size, and how the drift check compares the git working tree against what the in-flight tasks budgeted.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

An estimate is an opaque `unit -> quantity` dict, and `units` is the allow-list
of legal unit names. Two per-unit tables bound the quantities:

```toml
[tasks.estimate]
units = ["files"]

[tasks.estimate.max]
files = 5

[tasks.estimate.min]
files = 1
```

Outside the range is a *validation error*, not a warning: an oversized task must
be split before the work starts, not after. Terminal tasks are exempt.

A file count is the estimate `mt check` can actually verify. `[tasks.estimate.drift]`
compares the git working tree against the budget the in-flight tasks declare and
warns — never fails — when the tree has outgrown it:

```toml
[tasks.estimate.drift]
unit          = "files"              # estimate unit read as the file budget
in-flight     = 'status == "wip"'    # which tasks the diff is attributed to
line-multiple = 200                  # secondary line ceiling per declared file; 0 disables
message       = "rescope before going further"   # replaces the default advice
exclude       = ["docs/generated/**", "*.lock"]  # paths the file count skips
```

`exclude` names paths the file count never counts — generated output nobody
hand-edits, so touching it should not spend part of a budget meant for the
source that produced it. Each pattern is matched against the path git reports,
repo-root-relative (the root `mt` runs against, not the task root), with the
same glob syntax as `[tasks.roots] derive-tags`: `*` matches within one path
segment, `**` crosses directory boundaries. It only filters the file count —
the line total stays unfiltered, already documented below as a generous signal
that tolerates exactly this kind of distortion.

The working tree carries one diff and nothing in it says which file belongs to
which task, so attribution is by count — against the one in-flight task's
budget, or against the sum when several are open:

```
warning: working tree touches 14 file(s), over the 2 declared by `alpha` — split the remaining work into a new task and finish the part in hand
warning: working tree touches 16 file(s), over the 6 declared in total by 3 in-flight tasks — split the remaining work into a new task and finish the part in hand
  in flight: alpha, beta, gamma
```

The line total is a secondary soft signal at a generous multiple of the
declared file count, never a budget of its own: line counts are distorted by
generated files, lockfiles and formatting passes.

#callout("note")[
  The check is inert until an in-flight task declares an estimate in `unit`, and
  reads the tree strictly read-only (`git --no-optional-locks … status
  --porcelain`). Outside a git repository, or with git unavailable, it says
  nothing at all — it never guesses, and never fails the command.
]
