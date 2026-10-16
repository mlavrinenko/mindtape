#import "../utils/page.typ": page
#import "../utils/ui.typ": capture-line, exit-code, terminal
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/named-sets/",
  label: "Views & dictionaries",
  title: "Views & dictionaries",
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
  ("dict-teams", "the values an `x-` field takes"),
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

If you keep typing the same filter, give it a name. `[views.<name>]` names a
#link(u("/filters/"))[filter] together with a sort and a set of columns, so
you can ask for the listing by name:

#raw(
  render-slices(slices, view-parts, comment: "#"),
  lang: "toml",
  block: true,
)

Every key is optional. `filter` is a filter expression, and an empty one
matches every task. `sort` takes the same specs as `--sort`, and `columns` the
same column names as `--columns`. The #link(u("/cli/ls/"))[`mt ls` page]
lists both. `mt views <name>` prints the `description`.

`mt ls @name` applies the view's filter, sort and columns. Inside a larger
expression, `@name` is just the view's filter, as in
`mt ls '@waiting && kind == "feat"'`. If no
#link(u("/cli/ls/#conditional-rules"))[`[[ls.rules]]`] entry matches, a bare
`mt ls` shows the view named `default`, or the one `[ls] default-view`
names.

#terminal("named-sets-views")

`mt` checks every view when it loads the config. Each `filter` must parse,
each `sort` and `columns` entry must exist, each `@name` must name a view, and
no view may reach itself through `@name`.

= Dictionaries

When several rules and allow-lists need the same list of words, keep it in
one place. `[dicts.<name>]` names a set of strings that rules, filters and
allow-lists share:

#raw(
  render-slices(slices, dict-parts, comment: "#"),
  lang: "toml",
  block: true,
)

A dictionary takes entries from three keys, and needs at least one. `values`
lists them inline. `path` names a file with one entry per line, relative to
the project root. `mt` trims each line and skips blank lines and lines
starting with `#`. `include` adds every entry of other dictionaries, your own
or the built-ins.

`case` decides how a lookup compares. `fold`, the default, ignores ASCII case,
so the title word `Rotate` matches the entry `rotate`. `exact` compares bytes.
An included dictionary's entries compare under the including dictionary's
`case`.

`mt` ships two dictionaries you can use without declaring them:

- `mindtape-verbs`: the 500 most frequent English verbs, in base form.
- `mindtape-dev-verbs`: verbs software work uses that the frequency list
  leaves out, such as `refactor`, `deprecate` and `rate-limit`.

Your own dictionary names can't start with `mindtape-`, because that prefix
is for the built-ins.

`mt dicts` lists every dictionary with its entry count and case, the
built-ins included. `mt dicts <name>` prints one dictionary's entries, one per
line, after `include`, `values` and `path` are merged. That's the list a rule
reading the dictionary accepts, and a `fold` dictionary prints it lowercase:

#terminal("named-sets-dicts")

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
completion and the generated schema all see a plain list. The list keeps its
literal entries in order, adds each dictionary's entries in alphabetical
order, and drops repeats. An entry starting with `@` always names a
dictionary, so no tag or field value can start with `@`. `mt` refuses a tag
outside the list and exits #exit-code("named-sets-allowed"):

#terminal("named-sets-allowed")

When a rule refuses a task because `in_dict()` missed, the refusal says which
word the dictionary lacks and names up to three of its nearest entries, as in
#capture-line("start-titles-with-a-verb", "^hint: '"). A second hint runs
`mt dicts` on that dictionary, so you can read the whole list.

`mt` checks every dictionary when it loads the config. Each must have a
source and resolve to at least one entry. Each `path` must be readable. Each
`include` and `@<dict>` must name a dictionary, and for an unknown name `mt`
suggests the closest match. No dictionary may include itself, directly or
through others.
