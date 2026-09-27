#import "../utils/page.typ": page
#import "../utils/ui.typ": callout
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/estimates/",
  label: "Estimates",
  title: "Estimate caps and working-tree budgets",
  summary: "How tasks.estimate declares the legal units, and how plain rules cap a declared estimate and hold the git working tree to it.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Both blocks below are slices of examples/estimates/.mindtape/config.toml,
// the project `just check-examples` validates: the fixture's own two tasks sit
// against these caps, so a ceiling shown here is one that fires.
#let parts = (
  (
    "caps",
    "the unit allow-list, a ceiling gated on `difficulty`, a mechanical ceiling and a floor",
  ),
  (
    "budget",
    "the working-tree budget, counting everything but generated output",
  ),
)

#let slices = checked-slices(
  json("../generated/estimates-config.json"),
  "estimates-config",
  parts.map(p => p.at(0)),
)

= Units

An estimate is an opaque `unit -> quantity` dict. `[tasks.estimate]` carries
two keys: `units`, the allow-list of legal unit names, and `required`. The
allow-list is checked on open tasks only: a terminal task keeps whatever unit
it was estimated in, so retiring a unit is one edit to `units` rather than a
rewrite of every closed task that ever carried it.

No quantity is checked unless a project asks. A cap and a budget are ordinary
#link(u("/config/"))[`[[tasks.rules]]`] entries reading one unit as a number,
`estimate.<unit>` (see #link(u("/filters/"))[Filter language]).

= Caps

#raw(
  render-slices(slices, parts.slice(0, 1), comment: "#"),
  lang: "toml",
  block: true,
)

At `severity = "error"` a cap is a validation error, so every write command
refuses an oversized task before it lands: the task is split before the work
starts, not after. `!terminal` in `when` exempts a finished task, which can no
longer be split. A breach names the rule and carries its `message`:

```
error: tasks/rewrite-loader.typ:3: rules.judgment-stays-small: judgment work stays small; split it rather than widen the cap
```

= Working-tree budget

A file count is the estimate a rule can hold against git. `changed()` counts
the paths git reports changed, and a rule compares that against what the task
in flight declared:

#raw(
  render-slices(slices, parts.slice(1, 2), comment: "#"),
  lang: "toml",
  block: true,
)

`!`-prefixed globs name what the count skips — generated output nobody
hand-edits should not spend part of a budget meant for the source that
produced it. Globs match paths relative to the repository root, not the task
root. `has(estimate.files)` keeps the rule off a task that declares no budget.

The working tree carries one diff and nothing in it says which file belongs to
which task, so the rule holds each in-flight task to the whole change set; a
work-in-progress cap of one makes that exact. At `severity = "warn"` a breach
prints and `mt check` still exits zero: an unforeseen file is normal, and the
answer is a rescope, not a failed command.

#callout("note")[
  Git runs strictly read-only. Outside a git repository the rule reports once
  as skipped — a warning whatever its severity — and never fails the command.
]
