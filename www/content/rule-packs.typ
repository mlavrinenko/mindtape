#import "../utils/page.typ": page
#import "../utils/ui.typ": callout, terminal
#import "../utils/site.typ": u

#let meta = (
  route: "/rule-packs/",
  label: "Rule packs",
  title: "Rule packs",
  summary: "A rule pack is a project's task practice as one importable unit: extends adopts it, a same-named rule overrides one entry, disable drops one. What a pack file may contain, how it resolves, the merge order, and what mindtape-tasks enforces.",
)
#metadata(meta) <page-meta>

#show: page.with(..meta)

Copying a good `[[tasks.rules]]` / `[[tasks.body.headers.rules]]` /
`[[tasks.body.content.rules]]` practice into a new project by hand means
retyping every rule, and drifting from the original the moment either side
changes one. A rule pack is that practice as one importable unit:
#link(u("/config/"))[Configuration] covers `extends` and `disable` in a
paragraph under "Rule packs" — this page is the full account, plus what the
pack MindTape ships, `mindtape-tasks`, actually enforces.

= What a pack is

A pack is a config-syntax fragment carrying only the three rule arrays and
`[dicts]` — nothing else, and no nested `extends`. Loading one validates its
shape before a single rule reaches the merge: a pack declaring anything
outside `[[tasks.rules]]`, `[[tasks.body.headers.rules]]`,
`[[tasks.body.content.rules]]` or `[dicts]` — `[project]`, a stray
`[tasks.priority]`, or `extends` itself — is refused by name:

```text
pack `<label>` may not declare `<key>`
```

`<label>` is the pack's own name or path, so the message always points back
to the file that broke the rule rather than the layer that pulled it in.
Refusing `extends` inside a pack is what keeps resolution one level deep: a
layer's `extends` list can never chain into a pack that itself reaches for a
third one.

= Where a pack comes from

`extends` takes a list of entries, resolved independently. A *bare name* — no
path separator anywhere in it — looks up an embedded pack compiled into `mt`.
The `mindtape-` prefix is reserved for these; `mindtape-tasks` is the only one
that ships today. An unknown bare name is refused naming what IS available:

```text
extends: unknown pack `<name>`; available packs: <list>
```

Anything carrying a path separator — `"../shared/team-pack.toml"`,
`"./local-tasks.toml"`, even `"packs/mindtape-tasks.toml"` — resolves to a
TOML file instead, relative to the directory the declaring layer's OWN config
lives in (the project root, for the project's `.mindtape/config.toml`; the
equivalent ancestor directory for a `.mindtape/defaults.toml` or the XDG
global one). A missing path pack is refused naming the resolved path, not the
entry as typed, so a wrong relative path is easy to spot.

#callout("warn")[
  `"mindtape-tasks.toml"` with no separator is still a BARE name — and an
  unknown one, since the embedded registry's key is `mindtape-tasks`, not
  `mindtape-tasks.toml`. Add a `/` (even `"./mindtape-tasks.toml"`) to name a
  same-titled file on disk instead of the shipped pack.
]

= Adopting, overriding, and disabling a rule

Adopting one is one line, in any layer:

```toml
extends = ["mindtape-tasks"]
```

A same-named rule declared after it — in the same layer's own array —
replaces the pack's entry in place rather than sitting beside it. This
repo's own `.mindtape/config.toml` is the worked example: it extends
`mindtape-tasks` and then redeclares `section-allowlist` to add a local-only
`Dogfood` section the pack has no opinion on:

```toml
extends = ["mindtape-tasks"]

[[tasks.body.headers.rules]]
name = "section-allowlist"
action = "allowlist"
allowed = [
  { match = "^(Problem|Environment|TO-BE|Source of truth|Changes|Alternatives|Rollback|Out of scope|Dogfood)$" },
]
when = "!terminal"
message = "an open task's sections are Problem, TO-BE, Changes, Out of scope and Dogfood, plus its kind's own section"
```

Replacement by name works only ACROSS sources this way. Two entries sharing a
name WITHIN one layer's or one pack's own array both survive untouched, and
reach the rule's own validator as a duplicate-name config error instead of
one silently shadowing the other:

```text
tasks.body.headers.rules[3]: duplicate name `section-allowlist`, already used by tasks.body.headers.rules[0]
```

`disable` drops a rule outright instead of replacing it — useful for a rule a
pack ships that a project simply disagrees with:

```toml
extends = ["mindtape-tasks"]
disable = ["imperative-title"]
```

It reaches into any of the three arrays, inherited or extended, and a name
matching nothing anywhere is a config error rather than a silent no-op:

```text
disable: no rule named `<name>`
```

`[dicts]` needs neither mechanism: a pack's `[dicts]` table deep-merges ahead
of the extending layer's own, so a project overriding one dictionary by name
(the `mindtape-tasks` pack's own `[dicts.verbs]`, say) is just the normal
per-key table merge #link(u("/named-sets/"))[Named dictionaries] already
documents — last write wins, no array-splicing involved.

= Merge order

Per rule array, entries fold in one pass, in this order:

+ the four config layers, lowest precedence first (embedded floor, XDG
  global defaults, every ancestor `.mindtape/defaults.toml` farthest first,
  the project's own `.mindtape/config.toml` last);
+ within EACH layer, that layer's own `extends` packs first, in the order
  listed;
+ then that layer's own declared entries.

A later entry sharing an earlier one's `name` replaces it in place, so a rule
an ancestor `.mindtape/defaults.toml` declares — or a pack an early layer
extends — survives a nearer layer adding its own array, where it used to
vanish outright before packs existed (`extends`/`disable` are how the
wholesale-replace exception in #link(u("/config/"))[Configuration]'s "The
four layers" actually gets used). `disable` is applied once, after every
layer and pack has merged in — names from every layer's own `disable` list
are unioned first, so the same name declared twice is not an error twice.

= Reading `mt config show --origin`

`--origin` prints the contributing layers, then one `rule <label>: <origin>`
line per entry across the three arrays, in final merge order — `<label>` is
`tasks.rules.<name>` (or `.rules[<index>]` for an unnamed one), the same
name-or-position fallback a duplicate-name config error uses (above);
`mt check`'s own breach output names the same rule WITHOUT the leading
`tasks.` (`rules.<name>`, `body.headers.rules.<name>`), so grepping between
the two drops that one segment. `<origin>` is the layer kind
(`embedded`/`xdg`/`dir`/`project`) or `pack <name>` / `pack <path>`. Run in
THIS repo — which extends `mindtape-tasks` and overrides `section-allowlist`
exactly as shown above:

#terminal(read("../generated/rule-packs-origin.txt"))

Every header/content rule reads `pack mindtape-tasks` except
`section-allowlist` itself, which reads `project` — proof the override
replaced the pack's own entry rather than appending beside it — and the
`Dogfood`-specific rules this repo added on top, which the pack never
declared at all. `goals` (this repo's own `[[tasks.rules]]` entry) reads
`project` for the same reason: nothing here came from a pack.

= What `mindtape-tasks` enforces

`extends = ["mindtape-tasks"]` gives a project the six-section task shape
this repo's own backlog is written in: `section-allowlist` closes the
heading set to Problem, TO-BE, Changes, Out of scope, plus one kind-specific
section (`environment-for-fix`, `alternatives-for-feat`,
`source-of-truth-for-docs`, `rollback-for-dev`, each paired with a
`*-only-for-*` rule forbidding it on every other kind); a run of `*-after-*`
rules pins their relative order, and `no-subsections` forbids nesting a
heading under a heading at any depth. Content rules then check what each
section SAYS, not just that it exists: `problem-is-numbered-steps` and
`problem-marks-the-defect` require the Problem section as steps opening with
🔴, `to-be-has-green-bullets` / `to-be-bullets-all-green` require TO-BE as
bullets opening with 🟢, `changes-name-files` requires a backtick-quoted path
per bullet, `named-sections-non-empty` refuses an empty optional section, two
word caps (`section-word-cap`, `changes-word-cap`) push an over-long section
back into two tasks, `no-em-dash-pileup` caps em-dashes per paragraph, and
`show-shape-when-gating` demands a code block from a task tagged `gating`.
One more rule outside the body: `imperative-title` requires the title's first
word to be a verb, against the `verbs` dictionary the pack's own `[dicts]`
entry builds from the two built-in word lists.

That is prose ABOUT the pack — every name above is checked against the
file itself below, quoted in full so this page cannot drift from it as the
practice evolves:

#raw(read("../generated/mindtape-tasks-pack.toml"), lang: "toml", block: true)
