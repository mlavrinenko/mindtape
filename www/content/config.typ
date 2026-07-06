#import "../utils/page.typ": page
#import "../utils/ui.typ": terminal, callout
#import "../utils/site.typ": u

#show: page.with(
  title: "Configuration",
  active: "/config/",
  summary: "How .mindtape/config.toml overlays a defaults floor, the four-layer merge, and the sections you can set.",
)

`.mindtape/config.toml` overlays a shipped defaults floor. Omit a section to
inherit the floor's sane defaults, declare it to override, or set it empty (e.g.
`status = []`) to leave that aspect unconstrained.

= The four layers

Config is deep-merged in precedence order, lowest first:

+ the embedded floor (shipped with `mt`);
+ XDG global defaults — `~/.config/mindtape/defaults.toml`;
+ every `.mindtape/defaults.toml` walking up from the project root (farthest
  ancestor first, nearest wins);
+ the project's own `.mindtape/config.toml` (highest precedence).

Inspect the result:

```sh
mt config show            # effective config as re-parseable TOML
mt config show --origin   # which layer contributed each value, and from where
mt config show --defaults # just the embedded floor
mt config show --json     # effective config as JSON
```

= Sections

#table(
  columns: 2,
  table.header([Section], [Purpose]),
  [`[tasks] roots`], [Directories holding tasks. Table-form `[[tasks.roots]]` adds a display name and folder-tag policy — see #link(u("/derived-tags/"))[Derived tags].],
  [`[[tasks.status]]`], [The status legend, in lifecycle order: `value`, optional `glyph` / `color`, and `terminal`.],
  [`[tasks.priority]`, `[tasks.difficulty]`], [Numeric scales, bands, and frameworks. See #link(u("/priority/"))[Priority].],
  [`[tasks.estimate]`], [Allowed estimate `units` (free labels, never converted).],
  [`[tasks.tags]`], [`allowed` — the tag allow-list, checked against authored and derived tags.],
  [`[ls]`, `[rm]`], [Default filters, columns, and ordering. See #link(u("/filters/"))[Filter language].],
  [`[templates]`], [Template `dirs` and the `default` template. See #link(u("/templates/"))[Templates & slots].],
  [`[add]`, `[go]`], [Editor, interactive mode, and header for `mt add` / `mt go`.],
)

= Example

A compact project config — a status legend, a bounded priority scale, and allowed
estimate units:

```toml
[tasks]
roots = ["tasks"]

[[tasks.status]]
value = "wip"
glyph = "🔨"
color = "yellow"
[[tasks.status]]
value = "done"
glyph = "✅"
color = "green"
terminal = true

[tasks.priority]
kind = "int"
min  = 0
max  = 9
required = "non-terminal"

[tasks.estimate]
units = ["minutes", "hours", "days", "weeks"]

[ls]
default-filter = "status != \"done\""
```

#terminal("$ mt config show --origin
# [tasks.priority] max = 9        ← embedded floor
# [ls] default-filter            ← .mindtape/config.toml")

#callout("note")[
  `mt init` writes a slim scaffold that inherits the floor; `mt init --full`
  writes every section, spelled out and commented.
]
