#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": REPO, mindtape, u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/packs/",
  label: "Packs",
  title: "Packs",
  summary: "How a project adopts packs, how their keys and rules merge, and every rule the `mindtape:tasks` pack ships.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Running several projects the same way? Put the shared config in a pack. A
pack is a TOML file in config syntax that a project adopts by name: its rules,
its priority framework, its views, whatever you'd otherwise copy around. Each
project can still change any of it locally.
#link(u("/config/"))[Configuration] lists the `extends` and `disable` keys.
This page covers how `mt` finds and merges a pack, and lists the rules of the
pack `mt` ships, `mindtape:tasks`.

= Adopt a pack

`extends` in any #link(u("/config/#layers"))[config layer] names the packs
that layer adopts. If the layer declares a rule with the same `name` as a
pack's rule, it replaces that rule where the pack put it. #mindtape's own
`.mindtape/config.toml` adopts `mindtape:tasks` and replaces
`section-allowlist` to allow one more section, `Dogfood`:

#let parts = (
  ("extends", "the adoption, at the top of the file"),
  (
    "section-allowlist",
    "the replacement, further down: the name matches, the position does not matter",
  ),
)

#let slices = checked-slices(
  json("../generated/packs-config.json"),
  "packs-config",
  parts.map(p => p.at(0)),
)

#raw(render-slices(slices, parts, comment: "#"), lang: "toml", block: true)

A name replaces a rule only across sources. Two rules with one name in the
same file are a config error, as #link(u("/config/"))[Configuration]
describes.

= What a pack may hold

Any config key, except six. `extends` and `disable` belong to the project,
so a pack can't extend another pack and there's only ever one level to
resolve. `project.code` is the project's own name. `tasks.roots`,
`templates.dirs` and `[typst]` point into the project's own tree, which a
shared file can't know. `mt` refuses a pack that declares one of them and
names the pack and the key:

#terminal("packs-shape")

= Where a pack comes from

An `extends` entry with no `/` in it is `<source>:<pack>`. `mindtape` is the
one source `mt` knows without a declaration, and it holds the packs built into
`mt`, so far `mindtape:tasks`. `mt` refuses an unknown name and lists the packs
the source has:

#terminal("packs-unknown")

An entry with a `/` is a path to a TOML file, like `./team.toml` above. A
path is relative to the directory two levels above the config file that names
it. For `.mindtape/config.toml` that's the project root. For the user layer
`mindtape/defaults.toml` it's your config directory, such as `~/.config` on
Linux. `mt` refuses a missing file and prints the path it resolved.

#callout("warn")[
  `"tasks.toml"` has no `/`, so `mt` reads it as a pack name, not a file, and
  refuses it as unknown. Write `"./tasks.toml"` for a file of that name.
]

= Drop a rule

To turn off a rule you don't want, list its name in `disable`. It drops the
rule from any of the four arrays, whichever pack or layer declared it. A name
that matches no rule is a config error, so a typo can't leave a rule
running:

#terminal("packs-disable")

= Merge order

`mt` merges each config layer in turn, and each layer's packs merge just below
it. Tables merge key by key, and an array or a scalar replaces the value under
it whole, the same as between two layers. So a project's `[tasks.priority]`
keeps the framework a pack picked, and the floor fills in what neither sets:

#terminal("packs-merge")

A pack beats every layer below the one adopting it, and that layer beats the
pack. A #link(u("/named-sets/"))[dictionary] merges the same way, key by key.

Two packs in one `extends` that set one value differently are an error, since
nothing would say which one the project got. `mt` names the key and both
packs:

#terminal("packs-conflict")

Set that key in the layer that adopts both packs and the clash is gone, since
the layer's value wins anyway. Equal values agree, so two packs can both carry
the same dictionary. Packs adopted by two different layers never clash: the
higher layer wins.

The four rule arrays merge by name instead. Each takes the rules of the
layer's packs in `extends` order, then the layer's own rules, and a rule
replaces an earlier one with the same `name` in its place. After every layer
is merged, `mt` drops the rules named in any layer's `disable`. A name listed
in two layers is dropped once.

= Read `mt config show --origin`

To see where each rule came from, run `mt config show --origin`. It prints
the layers that contributed, then one line per rule in merge order:
`rule <label>: <origin>`. `<label>` is the rule's array and name, such as
`tasks.rules.<name>`, or `tasks.rules[<index>]` for a rule with no name.
`mt check` names a broken rule without the leading `tasks.`. `<origin>` is a
layer (`embedded`, `xdg`, `dir` or `project`), or `pack <name>` for a pack.
Here's the output in the #mindtape repository:

#terminal("packs-origin")

Every rule from `mindtape:tasks` reads `pack mindtape:tasks`, except
`section-allowlist`, which reads `project` because the project's rule replaced
it. The repository's own rules, such as `goals` and the `Dogfood` rules, read
`project` because only the project declares them. To read each rule's
severity and message as well, run #link(u("/cli/rules/"))[`mt rules`].

When `mt` reports a problem with a rule itself, not with a task, it points at
the rule's own `[[tasks.rules]]` header, whichever layer or pack declared it.
That's a file and line for a layer or a path pack, and `pack <name>:<line>`
for a built-in pack. `mt check` reports a rule skipped for lack of git this
way.

= The rules in `mindtape:tasks`

`extends = ["mindtape:tasks"]` gives you the task shape #mindtape's own
backlog uses. Each open task has four sections in order: Problem, TO-BE,
Changes and Out of scope. A `fix`, `feat`, `docs` or `dev` task adds one more
section of its own. The tables below are read from
#link(REPO + "/blob/main/crates/mindtape/src/packs/tasks.toml")[the
  pack file], and each message is the text `mt check` prints. The
`imperative-title` rule reads the `verbs` dictionary, which the pack builds
from the two word lists `mt` ships.

#let pack = toml("../generated/packs/tasks.toml")

#let rule-table(rules) = table(
  columns: (auto, auto, 1fr),
  table.header([Rule], [When], [Message]),
  ..rules
    .map(r => (
      raw(r.name),
      if "when" in r { raw(r.when) } else [always],
      r.at("message", default: ""),
    ))
    .flatten(),
)

== Task rules

#rule-table(pack.tasks.rules)

== Section rules

#rule-table(pack.tasks.body.headers.rules)

== Content rules

#rule-table(pack.tasks.body.content.rules)

== Commit rules

These run on a commit from `mt check --commit-msg` and on a range from
`mt check --commits`. See #link(u("/commit-rules/"))[Commit rules].

#rule-table(pack.commits.rules)
