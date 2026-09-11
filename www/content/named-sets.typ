#import "../utils/page.typ": page
#import "../utils/site.typ": u

#let meta = (
  route: "/named-sets/",
  label: "Views & dictionaries",
  title: "Named views and dictionaries",
  summary: "Two top-level config sections that name something once so every rule can point at it: a view bundles a filter with a sort and columns, a dictionary names a set of strings.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

= Named views

`[views.<name>]` bundles a filter with an optional sort and column set so
`mt ls` can reference it by name instead of repeating the expression:

```toml
[views.broken]
filter = 'blocked && stale(30)'

[views.summer]
filter = 'has_tag("summer") && !terminal'
sort = ["priority:desc", "due:asc"]
columns = ["#", "status", "priority", "title"]
description = "Summer push"
```

`filter` is required; `sort` (a list of `--sort`-grammar specs) and `columns`
(a subset of the `mt ls` column names — `#`, `status`, `ready`, `priority`,
`due`, `title`, `file`) are optional, as is a free-text `description`. Reference a
view with `@name`: bare `@name` applies filter, sort, and columns together;
`@name` inside a larger expression is filter-only, equivalent to
`in_view("name")` — see #link(u("/filters/"))[Filter language] for the
predicate and #link(u("/cli/"))[CLI reference] for `mt ls` / `mt views`.

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

```toml
[dicts.verbs]
values = ["add", "wire", "fix"]           # inline entries
path = "dicts/extra-verbs.txt"            # a file, one entry per line
include = ["mindtape-verbs", "mindtape-dev-verbs"]  # other dicts, unioned in
case = "fold"                             # fold (default) | exact
```

All three sources feed *one* set; any combination is legal and at least one
must be present. `path` is relative to the project root and may leave it (the
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
their own. A list may mix literal entries and `@<dict>` references freely; the
result is their union in declaration order, deduplicated (a repeated entry
counts once). An unknown dictionary name is refused at load, the same way an
unknown `include` target is: a nearest-match suggestion plus the list of known
dictionaries.

The `@` is the same sigil `mt ls` reads as a view reference (`@name`), and it
means the same thing here: always a dictionary, never a literal. There is no
escape for a tag or field value that itself starts with `@` — an entry like
`"@urgent"` is looked up as a dictionary named `urgent`, and fails the load
(as an unknown dictionary, with the same nearest-match treatment) unless a
dictionary by that name actually exists, in which case it expands to that
dictionary's entries rather than surviving as the literal string `"@urgent"`.

A filter reaches the same set with `in_dict("name", <string>)`, under that
dictionary's own `case` — see #link(u("/filters/"))[Filter language].

== The built-ins

Two dictionaries ship embedded in `mt` and need no declaration to be
referenced:

- `mindtape-verbs` — the 500 most frequent English verbs, base (imperative)
  form;
- `mindtape-dev-verbs` — 86 software-work verbs the general frequency list
  ranks too low to carry.

The `mindtape-` prefix is reserved for them: naming a project dictionary
`mindtape-anything` is a config error, so the namespace stays the tool's.

Dictionaries are validated at config load, once for the project rather than
once per task:

- the `mindtape-` prefix is refused on a project dictionary;
- each dictionary must name at least one source;
- every `include` must resolve — an unknown name gets a nearest-match
  suggestion and the list of known dictionaries;
- the `include` graph must be acyclic — a cycle is a config error;
- every `path` must resolve and read;
- a dictionary that resolves to no entries at all is a config error;
- every `@<dict>` reference in `[tasks.tags] allowed` or a
  `[tasks.fields.<name>] values` list must resolve too, with the same
  nearest-match treatment.
