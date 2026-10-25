#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/estimates/",
  label: "Estimates",
  title: "Estimates",
  summary: "Estimate units, file counts that queries back, and the rules that cap an estimate and compare it with git.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The config blocks are slices of examples/estimates/.mindtape/config.toml,
// and the task block a slice of its tasks, which `just check-examples`
// validates. www/generate-reference.sh runs the refused write and the budget
// check in a git copy of that project.
#let parts = (
  (
    "caps",
    "a ceiling for judgment work, a ceiling for mechanical work and a floor",
  ),
  ("queries", "the unit allow-list, and the unit that takes count(...)"),
  ("budget", "the working-tree budget, skipping generated files"),
)
#let task-slices = checked-slices(
  json("../generated/estimates.json"),
  "estimates",
  ("count",),
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

= Counting files with queries

A file count you guess is still a guess. `count(n, ..queries)` adds the files
your queries match to the `n` you'd write by hand, so the estimate tracks the
code as it is today. `[tasks.estimate.queries]` names the units that take it,
each with the path set its files must fall in, a glob or a
#link(u("/named-sets/"))[`@name` set]:

#raw(
  render-slices(slices, parts.slice(1, 2), comment: "#"),
  lang: "toml",
  block: true,
)

A task then writes the unit as a count:

#raw(
  render-slices(task-slices, (("count", "a manual 2, plus a grep"),)),
  lang: "typ",
  block: true,
)

Two files mention `HarvestCache` under `src/`, so `estimate.files` reads 4, and
every rule sees that number. The judgment cap below holds it to 5, so when the type
spreads to a fourth file, `mt check` refuses the task before you start it.

Two queries build the count:

- `grep(regex, within: glob)` matches the files whose text matches `regex`,
  in #link("https://docs.rs/regex/latest/regex/#syntax")[Rust regex syntax].
  Write the regex as a raw block, so a backslash needs no escape. `within`
  narrows it to the paths a glob or `@name` set admits.
- `glob(pattern)` matches the files whose path matches `pattern`.

A file several queries match counts once. Queries read the files git lists:
tracked ones and untracked ones `.gitignore` doesn't hide, with paths relative
to the repository root, as `changed()` reads them. `mt` matches them itself
and runs no shell, so the count comes out the same on every machine. A unit
with no `[tasks.estimate.queries]` entry takes a plain number only, and
`mt check` refuses a count there.

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
  render-slices(slices, parts.slice(2, 3), comment: "#"),
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
