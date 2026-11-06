#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("explain")

`mt explain` answers what this project enforces. Bare, it prints an index:
every rule grouped by its config section, the derived fields, and the tags,
views and dictionaries the project declares. A dictionary carries its entry
count, and `default` leads the view and tag lists when it exists. The index
ends on a `next:` line. It names `mt ls` for your work and `mt <command>
--help` for the flags of a command you have not met.

#cli-command("explain")

`--json` prints the index under a `schema_version`. Each listing keeps the
keys it printed under its old command, and the sections below say what they
hold.

= Rules

`mt explain rules` lists every rule your project checks, one line each. A line
holds the rule's severity, its name as `mt config show --origin` prints it, and
the message a breach prints. It lists what's left after packs, overrides and
`disable`, so a project that extends a #link(u("/packs/"))[rule pack] sees the
rules it actually got. A flip action that carries a `message` shows up too,
named by its place in `[[on.flip]]`.

A rule with no `severity` shows the level `mt` applies to it. That's `warn`
for `[[tasks.rules]]` and `[[commits.rules]]`, and `error` for a body rule. A
rule with no `message` shows the text a breach prints instead, or who words
it.

Under the rules, `mt explain rules` lists every derived field a
#link(u("/config/#derived-fields"))[collector] computes: its type, its
`derived.<name>`, its kind and origin, and the rules that read it. A Typst
collector names its module and function; an ast-grep collector names its
language, pattern and optional glob. `--for commit` omits that block, since a
commit rule cannot read a derived field.

`--for` keeps the rules one step checks. `add` lists what `mt add` checks
before it writes, `flip` adds the flip actions to that, and `commit` lists
your #link(u("/commit-rules/"))[commit rules]. `--json` adds each rule's
section, `when` and `assert`.

= Tags

`mt explain tags` lists every tag `[tasks.tags] allowed` permits, with the
rules that read it. A rule reads a tag when its `when` or `assert` reads
`has_tag("<tag>")` or `has(tags, "<tag>")`, however either is spaced, so a new
rule needs no registration. Each rule shows the message a breach prints, the
same line `mt explain rules` gives it.

Without `--for`, a tag that no rule reads is still a legal tag, so the listing
shows it with `area label` rather than dropping it. That covers tags a view or
a person reads, such as the area labels a project keeps for grouping.

`--for` keeps only the tags one step reads, by the same moments
`mt explain rules --for` uses. `add` keeps the tags read by the rules `mt add`
checks before it writes, `flip` adds the flip actions, and `commit` keeps the
commit rules. `--json` returns a `tags` map, each kept tag to its rules and
their messages, under `schema_version`.

= Views

`mt explain views` lists the project's views, or shows one by name. To define a
view, or to see what `@name` means in a listing, read
#link(u("/named-sets/"))[Views and dictionaries].

= Dictionaries

`mt explain dicts` lists the project's dictionaries, or one dictionary's
entries by name. That's the list a rule reading the dictionary accepts. To
declare a dictionary, or to see how its entries merge, read
#link(u("/named-sets/#dictionaries"))[Views and dictionaries].
