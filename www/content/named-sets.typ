#import "../utils/page.typ": page
#import "../utils/ui.typ": exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/named-sets/",
  label: "Views & dictionaries",
  title: "Named views and dictionaries",
  summary: "How views name a filter with its sort and columns, and dictionaries name a set of strings, for rules to share.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Every block is a slice of examples/named-sets/.mindtape/config.toml, which
// `just check-examples` validates. www/generate-reference.sh runs the
// listings in that project and the refused write in a copy.
#let view-parts = (
  ("view-default", "the listing a bare `mt ls` shows"),
  ("view-waiting", "a filter, a sort, columns and a description"),
)
#let dict-parts = (
  ("dict-verbs", "both built-in verb lists under one name"),
  ("dict-areas", "inline values, a file of entries, and the case rule"),
  ("dict-kinds", "inline values only"),
)
#let use-parts = (
  ("allow-lists", "allow-lists that read a dictionary"),
  ("rule", "a rule that reads a dictionary"),
)

#let slices = checked-slices(
  json("../generated/named-sets-config.json"),
  "named-sets-config",
  (view-parts + dict-parts + use-parts).map(p => p.at(0)),
)

= Views

`[views.<name>]` names a #link(u("/filters/"))[filter] together with a sort
and a set of columns, so a listing can be asked for by name:

#raw(
  render-slices(slices, view-parts, comment: "#"),
  lang: "toml",
  block: true,
)

Every key is optional. `filter` is a filter expression, and an empty one
matches every task. `sort` takes the specs `--sort` takes, and `columns` the
column names `--columns` takes, both listed on the
#link(u("/cli/"))[CLI reference]. `mt views <name>` prints the
`description`.

`mt ls @name` applies the view's filter, sort and columns. Inside a larger
expression, `@name` is the view's filter alone, as in
`mt ls '@waiting && kind == "feat"'`. When no `[[ls.rules]]` entry matches, a
bare `mt ls` shows the view named `default`, or the one `[ls] default-view`
names.

#terminal("named-sets-views")

`mt` checks every view when it loads the config. Each `filter` must parse,
each `sort` and `columns` entry must exist, each `@name` must name a view,
and no view may reach itself through `@name`.

= Dictionaries

`[dicts.<name>]` names a set of strings that rules, filters and allow-lists
share:

#raw(
  render-slices(slices, dict-parts, comment: "#"),
  lang: "toml",
  block: true,
)

A dictionary takes entries from three keys, and needs at least one. `values`
lists them inline. `path` names a file with one entry per line, relative to
the project root. `mt` trims each line and skips blank lines and lines
starting with `#`. `include` adds every entry of other dictionaries, the
project's own or the built-ins.

`case` decides how a lookup compares. `fold`, the default, ignores ASCII case,
so the title word `Rotate` matches the entry `rotate`. `exact` compares bytes.
An included dictionary's entries compare under the including dictionary's
`case`.

`mt` ships two dictionaries, which need no declaration:

- `mindtape-verbs`: the 500 most frequent English verbs, in base form.
- `mindtape-dev-verbs`: verbs software work uses that the frequency list
  leaves out, such as `refactor`, `deprecate` and `rate-limit`.

A project dictionary may not start with `mindtape-`, because the prefix names
the built-ins.

= Using a dictionary

`[tasks.tags] allowed` and `[tasks.fields.<name>] values` accept `@<dict>`
in place of an entry, and a rule reads one with
#link(u("/filters/"))[`in_dict()`]:

#raw(
  render-slices(slices, use-parts, comment: "#"),
  lang: "toml",
  block: true,
)

`mt` expands each `@<dict>` when it loads the config, so validation, `mt add`
completion and the generated schema all read a plain list. The list keeps
its literal entries in order, adds each dictionary's entries in alphabetical
order, and drops repeats. An entry starting with `@` always names a
dictionary, so no tag or field value can start with `@`. A tag outside the
list is refused, and `mt` exits #exit-code("named-sets-allowed"):

#terminal("named-sets-allowed")

`mt` checks every dictionary when it loads the config. Each must have a
source and resolve to at least one entry. Each `path` must be readable. Each
`include` and `@<dict>` must name a dictionary, and an unknown name gets the
closest match. No dictionary may include itself, directly or through
others.
