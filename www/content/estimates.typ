#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/estimates/",
  label: "Estimates",
  title: "Estimate bounds and working-tree drift",
  summary: "How tasks.estimate bounds a task's declared size, and how the drift check compares the git working tree against what the in-flight tasks budgeted.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Both blocks below are slices of examples/estimates/.mindtape/config.toml,
// the project `just check-examples` validates: the fixture's own two tasks sit
// against these bounds, so a ceiling shown here is one that fires.
#let parts = (
  (
    "bound",
    "the unit allow-list, a ceiling gated on `difficulty`, and a floor",
  ),
  ("drift", "every drift key, each at the value the fixture declares"),
)

#let slices = checked-slices(
  json("../generated/estimates-config.json"),
  "estimates-config",
  parts.map(p => p.at(0)),
)

= Bounds

An estimate is an opaque `unit -> quantity` dict, and `units` is the allow-list
of legal unit names. Two lists of per-unit entries bound the quantities, each
entry gated by an optional `when` filter and explained by an optional
`message`:

#raw(
  render-slices(slices, parts.slice(0, 1), comment: "#"),
  lang: "toml",
  block: true,
)

Outside the range is a *validation error*, not a warning: an oversized task must
be split before the work starts, not after. Terminal tasks are exempt.

Resolution is per unit: a unit's bound comes from the first entry, in config
order, whose `when` matches the task and that names the unit. An entry silent on
a unit leaves it to the entries below it, and a unit no matching entry names is
unbounded. A breach names the gate and carries the entry's `message` as its
advice:

```
rewrite-loader: estimate: 6 files is over the 5 allowed when `difficulty >= 3`; judgment work stays small; split it rather than widen the cap
```

The single-table spelling (`[tasks.estimate.max] files = 15`) still parses, as
one unconditional entry. A `when` that does not parse, an entry naming no unit,
and a unit outside a non-empty `units` list each fail the config at load.

= Working-tree drift

A file count is the estimate `mt check` can actually verify. `[tasks.estimate.drift]`
compares the git working tree against the budget the in-flight tasks declare and
warns — never fails — when the tree has outgrown it:

#raw(
  render-slices(slices, parts.slice(1, 2), comment: "#"),
  lang: "toml",
  block: true,
)

`exclude` names paths the file count never counts — generated output nobody
hand-edits, so touching it should not spend part of a budget meant for the
source that produced it. Each pattern is matched against the path git reports,
repo-root-relative (the root `mt` runs against, not the task root), with the
same glob syntax as `[tasks.roots] derive-tags`: `*` matches within one path
segment, `**` crosses directory boundaries. It only filters the file count.

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
