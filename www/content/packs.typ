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
packs `mt` ships: `mindtape:tasks`, `mindtape:dogfood`, `mindtape:flow`,
`mindtape:ice`, `mindtape:wip-caps`, `mindtape:budget`, `mindtape:tweaks` and
`mindtape:commits`.

= Adopt a pack

`extends` in any #link(u("/config/#layers"))[config layer] names the packs
that layer adopts. If the layer declares a rule with the same `name` as a
pack's rule, it replaces that rule where the pack put it. #mindtape's own
`.mindtape/config.toml` adopts `mindtape:tasks` and, after it,
`mindtape:dogfood`, whose `section-allowlist` replaces the tasks pack's to
allow one more section, `Dogfood`. It adopts the other packs below too,
`mindtape:budget` with `max-files` set to 5, and keeps only its tag list, its
actors, its path sets, its goal check and its close gate:

#let parts = (
  ("extends", "the adoption, at the top of the file"),
  (
    "section-allowlist",
    "the replacement, in the dogfood pack: the name matches, so it takes the place of the tasks pack's rule",
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
`mt`: `mindtape:tasks`, `mindtape:dogfood`, `mindtape:flow`, `mindtape:ice`,
`mindtape:wip-caps`, `mindtape:budget`, `mindtape:tweaks` and
`mindtape:commits`. `mt` refuses an unknown name and lists the packs
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

`mt` reads a source's packs from your cache directory, such as `~/.cache` on
Linux, under `mindtape/packs/<sha256>/`, named by the lock's hash.
`acme:team` is `team.toml` at the top of that tree, or `team/pack.toml` for a
#link(u("/packs/#a-pack-in-a-directory"))[folder] pack. A pack name takes letters, digits, `-` and `_`. `MINDTAPE_CACHE_HOME`
moves the cache, as #link(u("/config/#environment-variables"))[Configuration]
says.

Every load hashes the cached tree again and refuses one that doesn't match the
lock, naming its directory, so a file edited by hand there can't slip in. It
hashes each source once, however many of its packs you adopt. Every command
pays for that hash, so keep a pack source small, a repository that holds
packs and nothing else. A
pack from a source names its dictionary files only inside the source's tree.
Only `mt packs` touches the network, so every other command refuses a source
the cache lacks, with a hint to fetch it:

#terminal("packs-not-installed")

`mt packs install` fetches each locked source the cache lacks and refuses a
tree that doesn't hash to the lock. Like `npm ci`, it refuses a lock that
disagrees with `[pack-sources]`, so a CI job or a fresh clone gets the packs
you reviewed and nothing newer. In CI, run `mt packs install && mt check`.

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
the project. So `mt` refuses a `check` path that leaves the pack's directory. A rule and a dictionary that the project declares itself resolve
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
the rest of the task rules read `pack mindtape:tasks`. The exception is
`section-allowlist` and the Dogfood rules, which read `pack mindtape:dogfood`,
because that pack replaced the tasks pack's allowlist. The `stale-wip` hook
reads `pack mindtape:flow`, and the repository's own rule `goals` and its
`gates-fresh` hook read `project` because only the project declares them. Keys
that aren't rules read the same way, such as `key tasks.status` for
`pack mindtape:flow`. To read each rule's
severity and message as well, run #link(u("/cli/rules/"))[`mt rules`].

After the rules come the other keys a pack set, as `key <dotted path>:
<origin>`, and the parameters packs declare, as `param <name> = <value>: <set
by>, for pack <pack>`. A key names the layer instead of the pack when the layer
replaced the value, as `dicts.product.values` shows. `<set by>` is the layer
whose `with` set the parameter, such as `project` for `max-files`, or
`default` for a parameter nobody set.

When `mt` reports a problem with a rule itself, not with a task, it points at
the rule's own `[[tasks.rules]]` header, whichever layer or pack declared it.
That's a file and line for a layer or a path pack, and `pack <name>:<line>`
for a built-in pack. `mt check` reports a rule skipped for lack of git this
way.

= The rules in `mindtape:tasks`

`extends = ["mindtape:tasks"]` gives you the task shape #mindtape's own
backlog uses. Each open task has four sections in order: Problem, TO-BE,
Changes and Out of scope. A `fix`, `feat`, `docs` or `dev` task adds one more
section of its own. The pack also declares the `kind` field the rules read,
with `feat`, `fix`, `refactor`, `docs`, `test`, `dev` and `milestone`, and
requires it of an open task, and it forbids a comment in a task file. The
tables below are read from
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

= The rules in `mindtape:dogfood`

A project that builds with its own tool can ask each task how it will use the
change itself. `extends = ["mindtape:tasks", "mindtape:dogfood"]` adds a
Dogfood section after Out of scope. It must exist once, hold something ("Not
applicable" and a reason counts) and stay under 120 words. List it after
`mindtape:tasks`, since its `section-allowlist` replaces that pack's rule of
the same name, with `Dogfood` allowed, and a test keeps the two lists one name
apart.

#let dogfood = toml("../generated/packs/dogfood.toml")

#rule-table(dogfood.tasks.body.headers.rules)

#rule-table(dogfood.tasks.body.content.rules)

= The rules in `mindtape:flow`

`extends = ["mindtape:flow"]` gives you the six statuses of the floor, with the
moves between them. A task goes `proposed`, `accepted`, `wip`, then `done`,
`cancelled` or `failed`, and a closed task reopens into `wip`. `wip` and `done`
refuse a task whose `depends-on` target is unfinished. The pack also adds the
`stale-wip` hook, which refuses to start a task untouched for `days` days
until `mt review` stamps it. The default is 30 days.

#let flow = toml("../generated/packs/flow.toml")

#param-table(flow.params, (
  days: "How many days a task may sit untouched before starting it is refused.",
))

#table(
  columns: (auto, auto, 1fr),
  table.header([Status], [Moves to], [Blocked by an open dependency]),
  ..flow
    .tasks
    .status
    .map(s => (
      raw(s.value),
      if "to" in s { s.to.map(raw).join([, ]) } else [nothing],
      if s.at("unblocked", default: false) [yes] else [no],
    ))
    .flatten(),
)

A `[[tasks.status]]` of yours replaces the legend whole. The hook names `wip`,
so a legend with no `wip` is a config error until you list `stale-wip` in
`disable`. The pack leaves a close gate to the project: it would run a tool
your project may lack. See #link(u("/hooks/"))[Hooks] for writing one.

= The rules in `mindtape:ice`

`extends = ["mindtape:ice"]` scores priority with the ICE framework. It sets
`framework = "ice"`, `allow-manual = false`, `required = "!terminal"` and
`example = 3` under `[tasks.priority]`, so an open task calls `ice(...)` and a
raw number is refused. The pack sets nothing else: the shipped 0 to 9 scale and
its three bands show through, and a `[[tasks.priority.bands]]` of yours
replaces them. See #link(u("/priority/"))[Priority] for the frameworks.
