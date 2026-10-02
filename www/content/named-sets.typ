#import "../utils/page.typ": page
#import "../utils/site.typ": u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/named-sets/",
  label: "Views & dictionaries",
  title: "Named views and dictionaries",
  summary: "Two top-level config sections that name something once so every rule can point at it: a view bundles a filter with a sort and columns, a dictionary names a set of strings.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

// Both blocks below are slices of examples/learn-piano/.mindtape/config.toml,
// the project `just check-examples` validates and the docs generator runs `mt`
// against — so a section shown here is one that loads.
#let view-parts = (
  (
    "view-default",
    "a filter and nothing else; an omitted facet is simply not applied",
  ),
  ("view-stuck", "all four facets, reached as `mt ls @stuck`"),
)

#let dict-parts = (
  ("dict-verbs", "`include` alone: two built-in word lists under one name"),
  (
    "dict-composers",
    "inline `values`, a `path` file, and the `case` lookups compare under",
  ),
)

#let slices = checked-slices(
  json("../generated/named-sets-config.json"),
  "named-sets-config",
  (view-parts + dict-parts).map(p => p.at(0)),
)

= Named views

`[views.<name>]` names a query bundle — filter, sort, and column set — so
`mt ls` can reference it by name instead of repeating the expression:

#raw(render-slices(slices, view-parts, comment: "#"), lang: "toml", block: true)

Every facet is optional: an empty `filter` matches everything, `sort` is a
list of `--sort`-grammar specs, `columns` a subset of the `mt ls` column names
— #link(u("/cli/"))[CLI reference] lists both — and `description` is free
text shown by `mt views`. Reference a view with `@name`: bare `@name` applies
filter, sort, and columns together; `@name` inside a larger expression is
filter-only, equivalent to `in_view("name")` — see
#link(u("/filters/"))[Filter language] for the predicate.

Views are validated at config load:

- each `filter` must parse;
- `sort` and `columns` entries must be legal;
- every view reference must resolve;
- the reference graph must be acyclic — a cycle is a config error.

= Named dictionaries

`[dicts.<name>]` names a set of strings so more than one rule can share it. It
is a top-level section, sibling of `[views.<name>]`, for the same reason: a set
is not a property of tasks, and an allow-list, a filter and a future consumer
all read the same name.

#raw(render-slices(slices, dict-parts, comment: "#"), lang: "toml", block: true)

All three sources — `values`, `path` and `include` — feed *one* set; any
combination is legal and at least one must be present, which is what the two
entries above show between them. `path` is relative to the project root and may
leave it (the
same rule `[[typst.local-packages]] path` follows); blank lines and lines
starting with `#` are skipped, and each entry is trimmed. `include` names other
dictionaries — the project's own or a built-in — and unions their entries in
raw, so the *including* dictionary's `case` decides how they compare.

`case` decides lookups, not storage: `fold` (the default) is ASCII
case-insensitive, so a capitalised title word matches a lowercase entry;
`exact` is byte equality.

== Referencing a dictionary from an allow-list

`[tasks.tags] allowed` and `[tasks.fields.<name>] values` each accept `@<dict>`
anywhere they accept a literal entry:

```toml
[tasks.tags]
allowed = ["@areas", "swept", "size"]

[tasks.fields.kind]
values = ["@kinds"]
```

`@<dict>` expands to that dictionary's entries at config load — before any
rule reads the list — so validation, `mt add` slot completion and the
generated schema all see a plain, already-resolved set and need no change of
their own. A list may mix literal entries and `@<dict>` references freely:
the result is their union in list order, deduplicated (a repeated entry
counts once), except that a referenced dictionary contributes its entries
alphabetically — a dictionary carries no declaration order of its own.

The `@` is the same sigil `mt ls` reads as a view reference (`@name`), and it
means the same thing here: always a dictionary, never a literal. There is no
escape for a tag or field value that itself starts with `@`: an entry like
`"@urgent"` resolves as a dictionary named `urgent` — expanding to that
dictionary's entries if one exists, failing the load if none does — never
surviving as the literal string.

A filter reaches the same set with `in_dict("name", <string>)`, under that
dictionary's own `case` — see #link(u("/filters/"))[Filter language].

== The built-ins

Two dictionaries ship embedded in `mt` and need no declaration to be
referenced:

- `mindtape-verbs` — the 500 most frequent English verbs, base (imperative)
  form;
- `mindtape-dev-verbs` — software-work verbs the general frequency list ranks
  too low to carry, such as `update`, `refactor` and the hyphenated `rate-limit`.

The `mindtape-` prefix is reserved for them: naming a project dictionary
`mindtape-anything` is a config error, so the namespace stays the tool's.

Dictionaries are validated at config load, once for the project rather than
once per task:

- the `mindtape-` prefix is refused on a project dictionary;
- each dictionary must name at least one source;
- every `include`, and every `@<dict>` reference in `[tasks.tags] allowed`
  or a `[tasks.fields.<name>] values` list, must resolve — an unknown name
  gets a nearest-match suggestion and the list of known dictionaries;
- the `include` graph must be acyclic — a cycle is a config error;
- every `path` must resolve and read;
- a dictionary that resolves to no entries at all is a config error.
