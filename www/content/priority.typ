#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices
#import "../utils/framework-notes.typ": (
  anchors-table, bands-table, framework-table, scale-span,
)

#let meta = (
  route: "/priority/",
  label: "Priority",
  title: "Priority",
  summary: "How priority and difficulty work as numbers on a configured scale, set by hand or scored by a framework.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// The scales, bands, anchors and framework table come from
// www/generated/frameworks.json (www/generate-facts.sh). The two blocks are
// slices of examples/priority, which `just check-examples` validates, and
// www/generate-reference.sh runs the listing and the refused write there.
#let fw = json("../generated/frameworks.json")
#let config = checked-slices(
  json("../generated/priority-config.json"),
  "priority-config",
  ("framework",),
)
#let tasks = checked-slices(
  json("../generated/priority.json"),
  "priority",
  ("ice-call",),
)

A task's `priority` is a number on a scale. `mt` labels each number with the
band it falls in, and sorts and filters on the number. A task sets the number
by hand, such as `priority: 7`, or a framework computes it from typed inputs.

= Scale and bands

The shipped scale runs from #scale-span(fw.scales.priority). Each band is a
`[[tasks.priority.bands]]` entry:

#bands-table(fw)

`[tasks.priority]` sets the scale with `kind` (`int` or `real`), `min` and
`max`, and `required` takes a filter naming the tasks that must have a
priority. A project's bands replace the shipped list.
`mt config show --defaults` prints the shipped values, and #link(u("/config/"))[Configuration]
lists every key.

= Difficulty

`difficulty` is a second number with the same `kind`, `min`, `max` and
`required` keys, under `[tasks.difficulty]`. Its scale runs from
#scale-span(fw.scales.difficulty), and it is labelled by anchors instead of
bands. A value between two anchors takes the lower anchor's label:

#anchors-table(fw)

= Frameworks

A framework computes the priority from inputs a task fills in. This project
turns on ICE:

#raw(
  render-slices(
    config,
    (("framework", "score priority with ICE, and refuse a raw number"),),
    comment: "#",
  ),
  lang: "toml",
  block: true,
)

A task then calls the framework in place of a number:

#raw(
  render-slices(tasks, (("ice-call", "an ICE call"),)),
  lang: "typ",
  block: true,
)

`mt` computes the score and labels it with its band. `--sort priority` and a
filter such as `priority >= 7` read the score:

#terminal("priority-ls")

`allow-manual` decides whether a raw number is still accepted while a
framework is on. It defaults to `true`. With `false`, every task must call the
framework, so `mt` refuses a raw number and exits
#exit-code("priority-strict"):

#terminal("priority-strict")

#link(u("/templates/"))[Templates & slots] shows how `mt add` fills a
framework's inputs.

= Built-in frameworks

#framework-table(fw)

`[tasks.priority.frameworks.<name>]` declares a framework's inputs, with
their types and bounds, and its score formula or categories. The built-ins
are declared the same way in `crates/mindtape/src/frameworks.toml`, and a
project can override any of their keys. A task calls a project's own
framework with `framework("name", input: value, ...)`. A framework with
categories instead of a formula sorts each task into a category, and each
category has a fixed score on the scale.
