#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": REPO, mindtape, u
#import "../utils/slices.typ": checked-slices, render-slices

#let meta = (
  route: "/packs/",
  label: "Packs",
  title: "Packs",
  summary: "How a project adopts and tunes packs, how their rules merge, and the rules and parameters of the packs `mt` ships.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Running several projects the same way? Put the shared config in a pack. A
pack is a TOML file in config syntax that a project adopts by name: its rules,
its priority framework, its views, whatever you'd otherwise copy around. Each
project can still change any of it locally.
#link(u("/config/"))[Configuration] lists the `extends` and `disable` keys.
This page covers how `mt` finds and merges a pack, and lists the rules of the
packs `mt` ships: `mindtape:tasks`, `mindtape:wip-caps`, `mindtape:budget`,
`mindtape:tweaks` and `mindtape:commits`.

= Adopt a pack

`extends` in any #link(u("/config/#layers"))[config layer] names the packs
that layer adopts. If the layer declares a rule with the same `name` as a
pack's rule, it replaces that rule where the pack put it. #mindtape's own
`.mindtape/config.toml` adopts `mindtape:tasks` and replaces
`section-allowlist` to allow one more section, `Dogfood`. It adopts the four
packs below too, `mindtape:budget` with `max-files` set to 5:

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

Any config key, except seven. `extends` and `disable` belong to the project,
so a pack can't extend another pack and there's only ever one level to
resolve. `[pack-sources]` belongs to the project too, so every source sits in
one file. `project.code` is the project's own name. `tasks.roots`,
`templates.dirs` and `[typst]` point into the project's own tree, which a
shared file can't know. `mt` refuses a pack that declares one of them and
names the pack and the key:

#terminal("packs-shape")

`[params]` is the one key a pack holds that a layer doesn't. It declares the
pack's parameters, which `mt` writes into the pack's expressions and never
merges into the config.

= Where a pack comes from

An `extends` entry with no `/` in it is `<source>:<pack>`. `mindtape` is the
one source `mt` knows without a declaration, and it holds the packs built into
`mt`: `mindtape:tasks`, `mindtape:wip-caps`, `mindtape:budget`,
`mindtape:tweaks` and `mindtape:commits`. `mt` refuses an unknown name and lists the packs
the source has:

#terminal("packs-unknown")

An entry with a `/` is a path to a TOML file, like `./team.toml` above, or to
a #link(u("/packs/#a-pack-in-a-directory"))[folder]. A path is relative to the directory two levels above the config file that names
it. For `.mindtape/config.toml` that's the project root. For the user layer
`mindtape/defaults.toml` it's your config directory, such as `~/.config` on
Linux. `mt` refuses a missing file and prints the path it resolved.

#callout("warn")[
  `"tasks.toml"` has no `/`, so `mt` reads it as a pack name, not a file, and
  refuses it as unknown. Write `"./tasks.toml"` for a file of that name.
]

= Packs from a git source

Your team's packs can live in a git repository of their own. Declare it as
`[pack-sources.<name>]` in the project's `.mindtape/config.toml`, with `git`
for the repository and `ref` for a tag, branch or commit, and adopt a pack
from it as `<name>:<pack>`. Only the project config declares a source or adopts
a pack from one, so a clone resolves the same packs on every machine. The name
`mindtape` is taken.

`.mindtape/packs.lock` pins each source to the commit its `ref` named and a
hash of the files there, and you commit it beside the config. Every command
that reads the config checks the lock against the sources first. A source the
lock lacks, or pins from another `git` or `ref`, is refused, here after a bump
from `v1` to `v2`.

#terminal("packs-stale")

= A pack in a directory

A pack that carries files needs a folder. Name the folder in `extends`, and
`mt` reads `pack.toml` inside it. The folder's `templates/` joins
`[templates] dirs` behind your own directories, so `mt add -t feat` finds a
template the pack ships, and a template of yours with the same name wins.
`mt add --list` lists both:

#terminal("packs-dir")

The folder can sit anywhere, inside the project or not. When two packs ship a
template with one name, the pack of the later layer wins, and within a layer
the one listed last. If a `team.toml` sits beside a `team/` folder,
`extends = ["./team"]` could mean either, so `mt` refuses it and names both:

#terminal("packs-ambiguous")

Write `"./team.toml"` to read the file.

= Files a pack names

A pack can name files of its own: a rule's Typst `check`, or a
#link(u("/named-sets/"))[dictionary] `path`. Both resolve from the pack's
directory, which is the folder of a directory pack, or the one holding a pack
file. So a pack that sits in `team/` finds `team/check.typ` and
`team/words.txt` wherever the project lives, with nothing copied across.

A pack's check file runs in a Typst world rooted at that directory. It imports
the pack's other files and `@local` packages, and it can't import a file of
the project. A rule and a dictionary that the project declares itself resolve
from the project root, as they always did, and so does a rule that replaces a
pack's rule by name. The built-in packs name no files.

= Drop a rule

To turn off a rule you don't want, list its name in `disable`. It drops the
rule from any of the five arrays, whichever pack or layer declared it, a
named #link(u("/hooks/"))[flip hook] included. A name that matches no rule is a
config error, so a typo can't leave a rule running:

#terminal("packs-disable")

= Set a pack's parameters

A pack can leave a number open for each project to pick. It declares the
value with a default in `[params]`, and its expressions read it as
`param("name")`. A project sets its own value in the `extends` entry with
`with`, and still gets every later fix to the rule. Here the pack caps a task
at 5 files and the project raises the cap to 8:

#terminal("packs-params")

`mt` writes each value into the pack's expressions when it loads the config,
so `mt check`, `mt rules` and `mt config show` all see `8`. It works in every
expression the pack holds, from a rule's `when` and `assert` to a view's
`filter`, an `[[on.flip]]` `when` or `list`, each `required` and the `[ls]`
and `[rm]` filters. A `param()` inside a string literal stays as written, and so does
one in a rule's `message`, which is text and no expression. A bare entry like `"./team.toml"` keeps every
default.

A value is a string, a number or a boolean, of the same type as its default.
The parameters belong to their pack, so two packs can each have a
`max-files`. `mt` refuses a `with` key the pack doesn't declare and suggests
the nearest one:

#terminal("packs-params-unknown")

It also refuses a value of another type than the default, a `param()` that
names no parameter, and one pack listed twice in one `extends`, even by two
paths to the same file. A `param()`
in a layer's own expression is refused too, since only a pack has
parameters.

= A mistake in a pack

An error names the file that holds the mistake, with the line, so you open
the pack and not your own config. That goes for a path pack, a directory pack,
a source's pack and the `defaults.toml` of a parent folder alike. An unknown
key:

#terminal("packs-typo")

And a value that doesn't pass its check:

#terminal("packs-bad-value")

A problem that only the merge causes, with no one file to blame, leads with
the key alone.

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

Five arrays merge by name instead: `[[tasks.rules]]`, the two body rule
arrays, `[[commits.rules]]` and `[[on.flip]]`. Each takes the entries of the
layer's packs in `extends` order, then the layer's own, and an entry replaces
an earlier one with the same `name` in its place. A flip hook with no `name`
appends and nothing can replace it, so a pack's hook that the project should
be able to change needs one. After every layer
is merged, `mt` drops the rules named in any layer's `disable`. A name listed
in two layers is dropped once.

= Read `mt config show --origin`

To see where each rule, key and parameter came from, run `mt config show
--origin`. It prints the layers that contributed, then one line per rule in
merge order: `rule <label>: <origin>`. `<label>` is the rule's array and name, such as
`tasks.rules.<name>` or `on.flip.<name>`, or `tasks.rules[<index>]` for a rule
with no name.
`mt check` names a broken rule without the leading `tasks.`. `<origin>` is a
layer (`embedded`, `xdg`, `dir` or `project`), or `pack <name>` for a pack.
Here's the output in the #mindtape repository:

#terminal("packs-origin")

Every rule from a pack reads `pack <name>`: the two caps read
`pack mindtape:wip-caps`, `judgment-stays-small` and `src-budget` read
`pack mindtape:budget`, `tweak-lands-done` reads `pack mindtape:tweaks`, and
the rest of the task rules read `pack mindtape:tasks`. The exception is `section-allowlist`, which reads `project` because the project's rule replaced
it. The repository's own rules, such as `goals` and the `Dogfood` rules, read
`project` because only the project declares them. To read each rule's
severity and message as well, run #link(u("/cli/rules/"))[`mt rules`].

After the rules come the other keys a pack set and the parameters it
declares. A key line is `key <dotted path>: <origin>`, and it names the layer
instead of the pack when the layer replaced the value. A parameter line is
`param <name> = <value>: <set by>, for pack <pack>`, where `<set by>` is the
layer whose `with` set it, or `default`:

// docs:hand-typed this repository extends no pack that declares a parameter
```console
$ mt config show --origin
rule tasks.rules.small: pack /work/app/team.toml
key tasks.priority.framework: pack /work/app/team.toml
param max-files = 8: project, for pack /work/app/team.toml
```

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

#let param-table(params, notes) = table(
  columns: (auto, auto, 1fr),
  table.header([Parameter], [Default], [Meaning]),
  ..params
    .pairs()
    .map(((name, value)) => (raw(name), raw(str(value)), notes.at(name)))
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

= The rules in `mindtape:wip-caps`

`extends = ["mindtape:wip-caps"]` keeps one milestone and one other task open
at a time. Both rules are errors, so a flip that would open a second one
is refused. Each cap is a parameter, read from
#link(REPO + "/blob/main/crates/mindtape/src/packs/wip-caps.toml")[the pack file]:

#let caps = toml("../generated/packs/wip-caps.toml")

#param-table(caps.params, (
  milestones: "How many milestones may be `wip` at once.",
  leaves: "How many tasks that aren't milestones may be `wip` at once.",
))

#rule-table(caps.tasks.rules)

= The rules in `mindtape:budget`

`extends = ["mindtape:budget"]` holds a task's size to a number of source
files. A task estimates the files it means to touch in `estimate.src-files`.
`judgment-stays-small` refuses an open task of judgment work, and a milestone
is exempt, that estimates more than `max-files`. `src-budget` warns when the
working tree has changed more files than the task in progress estimated.

#let budget = toml("../generated/packs/budget.toml")

#param-table(budget.params, (
  max-files: "The most source files a judgment task may estimate.",
  min-difficulty: "The lowest difficulty that counts as judgment work.",
))

#rule-table(budget.tasks.rules)

The pack also sets `[tasks.estimate] units = ["src-files"]`, a query that
counts a task's `grep` matches inside the `source` path set, and a `[dicts.source]`
of `**`, so every file counts. Keys merge key by key, so your own
`[tasks.estimate.queries]` adds to the pack's. An array replaces it, so a
`[dicts.source]` of yours with `!` globs for docs and generated files replaces
the pack's `**`, as #mindtape's own does. A parameter reaches expressions only,
never the unit, so a project that sizes in `files` overrides both rules by name.

= The rules in `mindtape:tweaks`

A tweak is a change too small to plan: `mt add -t tweak` files it done, in the
commit that makes the change. `extends = ["mindtape:tweaks"]` holds a project
to that. `tweak-lands-done` refuses a tweak that isn't terminal, and
`tweak-stays-one-file` refuses a commit that closes a tweak and changes more
than one file of the `source` path set. The pack leaves the `tweak` tag to the
project: list it in `[tasks.tags]` if that table limits the tags. It sets
`[dicts.source]` to `**`, the same as `mindtape:budget`, so a project adopts
both and the two agree. Your own `[dicts.source]` replaces it for both.

#let tweaks = toml("../generated/packs/tweaks.toml")

#rule-table(tweaks.tasks.rules)

#rule-table(tweaks.commits.rules)

= The rules in `mindtape:commits`

`extends = ["mindtape:commits"]` holds each commit to two rules, and both
are #link(u("/commit-rules/"))[commit rules]. `changelog-with-product-change`
asks a commit that changes a file of the `product` path set for a
`CHANGELOG.md` entry. A `Changelog: none` trailer says there's nothing to
announce, and a `fixup!`, `squash!` or `amend!` subject is exempt, since
autosquash folds it into a commit that has one. `refs-task-is-started`
refuses a commit whose `Refs:` task isn't `wip` or terminal in the staged tree.

The pack sets `[dicts.product]` to `**`, so every file counts as product.
Replace it with your own `[dicts.product]`, such as `crates/**` and
`!**/*_tests.rs`, to say what ships to a user. A project that keeps its
changelog elsewhere overrides the rule by name.

#let commits = toml("../generated/packs/commits.toml")

#rule-table(commits.commits.rules)
