#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/estimates/",
  label: "Estimates",
  title: "Estimate caps and working-tree budgets",
  summary: "The estimate units `[tasks.estimate]` allows, and the rules that cap an estimate and compare it with the git tree.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Both blocks are slices of examples/estimates/.mindtape/config.toml, which
// `just check-examples` validates. www/generate-reference.sh runs the
// refused write and the budget check in a git copy of that project.
#let parts = (
  (
    "caps",
    "the unit allow-list, a ceiling for judgment work, a ceiling for mechanical work and a floor",
  ),
  ("budget", "the working-tree budget, skipping generated files"),
)

#let slices = checked-slices(
  json("../generated/estimates-config.json"),
  "estimates-config",
  parts.map(p => p.at(0)),
)

= Units

A task's estimate is a dict of units to numbers, such as
`estimate: (files: 5)`. Pick units you can check: files changed, say, rather
than hours. `[tasks.estimate] units` lists the unit names you allow, and
`required` says which tasks must carry an estimate. `mt` checks `units` on
open tasks only, so a finished task keeps its unit after you drop it from the
list.

`mt` doesn't check any quantity on its own. A cap or a budget is a
#link(u("/rules/"))[`[[tasks.rules]]`] entry that reads one unit as a number,
`estimate.<unit>`.

= Caps

#raw(
  render-slices(slices, parts.slice(0, 1), comment: "#"),
  lang: "toml",
  block: true,
)

`!terminal` in each `when` leaves finished tasks alone. A breach at
`severity = "error"` is a validation error, so every write command refuses it.
Here `mt set` turns a mechanical task estimated at 12 files into judgment
work, and the judgment cap of 5 refuses it. `mt` writes nothing and exits
#exit-code("estimates-cap"):

#terminal("estimates-cap")

For `mt add` refusing a task over a cap, see
#link(u("/split-oversized-tasks/"))[Split oversized tasks].

= Working-tree budget

A file count is an estimate a rule can compare with git, so you find out
when a task grows past its plan. #link(u("/filters/"))[`changed()`] counts the
paths git reports changed, and the rule compares that count with the estimate
of each task in progress:

#raw(
  render-slices(slices, parts.slice(1, 2), comment: "#"),
  lang: "toml",
  block: true,
)

A glob starting with `!` skips matching paths, so generated files don't use
up a budget meant for the source that produces them. Globs match paths
relative to the repository root. `has(estimate.files)` keeps the rule off a
task with no estimate.

Git reports one set of changes, and nothing in it says which file belongs to
which task. So the rule compares the whole set with each task in progress.
That's only exact when one task is in progress at a time (see
#link(u("/rules/"))[work-in-progress caps]). At `severity = "warn"` a breach
prints a warning and `mt check` still exits 0, because an unplanned file is
normal, and the fix is to rescope the task.

After `mt wip rewrite-loader` and five new files under `src/`, six paths
have changed, counting the task file, against an estimate of 5:

#terminal("estimates-budget")

#callout("note")[
  A rule that reads `changed()` needs a git repository. Outside one, `mt`
  reports the rule once as skipped. `mt` runs git read-only, at most once per
  command.
]
