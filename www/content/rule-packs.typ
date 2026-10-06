#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": REPO, u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/rule-packs/",
  label: "Rule packs",
  title: "Rule packs",
  summary: "How a project adopts, overrides and drops rule packs, and every rule the `mindtape-tasks` pack ships.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

A rule pack is a set of rules that a project adopts by name, so several
projects can share one task practice and each can change it locally.
#link(u("/config/"))[Configuration] lists the `extends` and `disable` keys.
This page says how `mt` resolves and merges a pack, and lists the rules of the
pack `mt` ships, `mindtape-tasks`.

= Adopt a pack

`extends` in any #link(u("/config/#layers"))[config layer] names the packs that
layer adopts. A rule the
layer declares with the same `name` as a pack's rule replaces that rule where
the pack put it. MindTape's own `.mindtape/config.toml` adopts
`mindtape-tasks` and replaces `section-allowlist` to allow one more section,
`Dogfood`:

#let parts = (
  ("extends", "the adoption, at the top of the file"),
  (
    "section-allowlist",
    "the replacement, further down: the name matches, the position does not matter",
  ),
)

#let slices = checked-slices(
  json("../generated/rule-packs-config.json"),
  "rule-packs-config",
  parts.map(p => p.at(0)),
)

#raw(render-slices(slices, parts, comment: "#"), lang: "toml", block: true)

A name replaces a rule only across sources. Two rules with one name in the
same file are a config error, as #link(u("/config/"))[Configuration]
describes.

A pack's `[dicts]` merge ahead of the layer's own, key by key, so a layer
replaces one #link(u("/named-sets/"))[dictionary] by declaring it under the
same name.

= What a pack may hold

A pack is a TOML file in config syntax that holds only rule arrays and
`[dicts]`: `[[tasks.rules]]`, `[[tasks.body.headers.rules]]`,
`[[tasks.body.content.rules]]`, `[[commits.rules]]` and `[dicts]`. `mt` refuses
a pack that declares anything else, `extends` included, and names the pack and
the key. Because a pack cannot extend another pack, resolution is one level
deep.

#terminal("rule-packs-shape")

= Where a pack comes from

An `extends` entry with no `/` in it is the name of a pack built into `mt`.
Names that start with `mindtape-` are kept for these, and `mindtape-tasks` is
the only one. `mt` refuses an unknown name and lists the packs it has:

#terminal("rule-packs-unknown")

An entry with a `/` is a path to a TOML file, as `./team.toml` is above. A
path is relative to the directory two levels above the config file that names
it. For `.mindtape/config.toml` that is the project root, and for the user
layer `mindtape/defaults.toml` it is your config directory, such as `~/.config`
on Linux. `mt` refuses a missing file and prints the path it resolved.

#callout("warn")[
  `"mindtape-tasks.toml"` has no `/`, so `mt` reads it as the name of a built-in
  pack, and refuses it as unknown. Write `"./mindtape-tasks.toml"` for a file
  of that name.
]

= Drop a rule

`disable` lists rule names to drop. It drops a rule from any of the four
arrays, whichever pack or layer declared it. A name that matches no rule is a
config error, so a typo cannot leave a rule running:

#terminal("rule-packs-disable")

= Merge order

`mt` merges each config layer in turn. Within a layer,
each rule array takes the rules of the layer's `extends` packs in the order
listed, then the layer's own rules. After every layer is merged, `mt` drops
the rules named in any layer's `disable`. A name listed in two layers is
dropped once.

= Read `mt config show --origin`

`--origin` prints the layers that contributed, then one line per rule in merge
order: `rule <label>: <origin>`. `<label>` is the rule's array and name, such
as `tasks.rules.<name>`, or `tasks.rules[<index>]` for a rule with no name. `mt check` names a broken rule without the leading `tasks.`. `<origin>`
is a layer (`embedded`, `xdg`, `dir` or `project`), or `pack <name>` for a
pack. This is the output in the MindTape repository:

#terminal("rule-packs-origin")

Every rule from `mindtape-tasks` reads `pack mindtape-tasks`, except
`section-allowlist`, which reads `project` because the project's rule replaced
it. The repository's own rules, such as `goals` and the `Dogfood` rules, read
`project` because only the project declares them.

A report about a rule, rather than about a task, points at the rule's own
`[[tasks.rules]]` header, whichever layer or pack declared it: a file and line
for a layer or a path pack, and `pack <name>:<line>` for a built-in pack.
`mt check` reports a rule skipped for lack of git this way.

= The rules in `mindtape-tasks`

`extends = ["mindtape-tasks"]` gives a project the task shape MindTape's own
backlog uses. Each open task has four sections in order: Problem, TO-BE,
Changes and Out of scope. A `fix`, `feat`, `docs` or `dev` task adds one more
section of its own. The tables below are read from
#link(REPO + "/blob/main/crates/mindtape/src/packs/mindtape-tasks.toml")[the pack file], and each message is the text `mt check` prints. The
`imperative-title` rule reads the `verbs` dictionary, which the pack builds
from the two word lists `mt` ships.

#let pack = toml("../generated/mindtape-tasks-pack.toml")

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
`mt check --commits`. #link(u("/commit-rules/"))[Commit rules] says how.

#rule-table(pack.commits.rules)
