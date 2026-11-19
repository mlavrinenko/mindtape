#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("explain")

`mt explain` answers what this project is and what it enforces. Bare, it opens
on the project's shape, in blocks. The `project:` block lists the task roots
with the tag each emits, the status legend split into `open` and `terminal`,
and the actors with their kind and roles. The `task:` block lists the priority
scale and the framework that scores it, each input with its own bounds and the
scale the score lands on. It also lists the difficulty scale, the estimate
units, the `kind` values and the link kinds. Then come the `[tasks.comments]
policy`, the `x-` extensions with the rule that reads each, and the templates
with the one `mt add` picks.

Each block carries the rules that check it. `task:` holds `rules`, and
`body.headers` and `body.content` under it. `commits:` holds `rules`, and
`add:`, `flip:` and `rm:` hold the hooks each command runs. `rm:` also holds
the `[rm] default-filter` as `filter:`. A section prints one line per severity
it holds, `error` before `warn`. A reader meets what refuses the work first.

A key the project leaves open prints what its absence permits: `any` for a
free vocabulary, `none` for an empty registry, and the policy word for
comments. Every line reads the config alone. The index doesn't load a task,
count anything or evaluate a filter.

A gate the user has not allowed prints in a `refusals:` block, one line per
command: the section that declares it, the command, and the `mt allow` that
records the approval. That is each `[[tasks.waits]]` checker and each
`[[on.flip]]` or `[[on.rm]]` `run`. A project with every command allowed
doesn't print the block at all.

Under the blocks, the index prints the derived fields and the tags, views and
dictionaries the project declares. A dictionary shows its entry count, and
`default` comes first in the view and tag lists when it exists. A blank line
separates the index from a `more:` line naming the listings that go deeper:
`mt explain <add|rules|tags|views|dicts|templates>`.

#cli-command("explain")

`--json` prints the index as one object whose first key is `$schema`, the URL
of the schema for that object. The shape sits under `project`, `task` and
`commits`, the keys it leaves out under `absent`, the refused gates under
`refusals`, the add hooks under `add`, and the listings under `rules`,
`derived`, `tags`, `views` and `dicts`. Each section under `rules` keeps its
config path and maps a severity to its rule names, and each entry under
`task.priority.inputs` carries its `kind`, `min`, `max` and `values`. The
sections below say what they contain.

= Add hooks

`mt explain add` lists the #link(u("/hooks/#add-hooks"))[`[[on.add]]`] hooks
your project runs, in declaration order. Each line gives the hook's label
(`on.add.<name>`, or `on.add[<index>]` when unnamed), its `when`, its actions
and the layer or pack that supplied it. A hook a nearer layer replaced or
`disable` dropped is absent, so the list is what `mt add` actually runs.
`--json` wraps the hooks in `hooks` under `$schema`, each with the same
`label` beside its `name`.

= Rules

`mt explain rules` lists every rule your project checks, one line each. Each line
gives the rule's severity, its name as `mt config show --origin` prints it, and
the message a breach prints. It lists what's left after packs, overrides and
`disable`, so a project that extends a #link(u("/packs/"))[rule pack] sees the
rules in effect. A flip or rm gate shows up with or without a `message`, and
any other hook action shows up only when it has one. Each takes its name from
its place in `[[on.flip]]` or `[[on.rm]]`.

A rule with no `severity` shows the level `mt` applies to it. That's `warn`
for `[[tasks.rules]]` and `[[commits.rules]]`, and `error` for a body rule. A
rule with no `message` shows the text a breach prints instead, or names the
source that supplies it.

Under the rules, `mt explain rules` lists every derived field a
#link(u("/config/#derived-fields"))[collector] computes: its type, its
`derived.<name>`, its kind and origin, and the rules that read it. A Typst
collector gives its module and function. An ast-grep collector gives its
language, pattern and optional glob. `--for commit` omits that block, since a
commit rule cannot read a derived field.

`--for` keeps the rules one step checks. `add` lists what `mt add` checks
before it writes, `flip` adds the flip actions to that, `rm` lists the
#link(u("/hooks/#removal-hooks"))[removal hooks], and `commit` lists your
#link(u("/commit-rules/"))[commit rules]. `--json` names its schema in
`$schema`, then adds each rule's section, `when` and `assert`.

= Tags

`mt explain tags` lists every tag `[tasks.tags] allowed` permits, with the
rules that read it. A rule reads a tag when its `when` or `assert` reads
`has_tag("<tag>")` or `has(tags, "<tag>")`. Spacing doesn't matter, and a new
rule doesn't need registration. Each rule shows the message `mt explain rules`
gives it, `(no message set)` included.

Without `--for`, a tag that every rule skips is still a legal tag. The listing
shows it with `area label` rather than dropping it. That covers tags a view or
a person reads, such as the area labels a project keeps for grouping.

`--for` keeps only the tags one step reads, for the same steps
`mt explain rules --for` uses. `add` keeps the tags read by the rules `mt add`
checks before it writes, `flip` adds the flip actions, `rm` keeps the removal
hooks, and `commit` keeps the commit rules. `--json` returns a `tags` map from each kept tag to its rules and
their messages, with its own `$schema`.

= Views

`mt explain views` lists the project's views, or shows one by name. `--json`
wraps the list in `views` under `$schema`. One named view prints flat under
`$schema`. To define a view, or to see what `@name` means in a listing, read
#link(u("/named-sets/"))[Views and dictionaries].

= Dictionaries

`mt explain dicts` lists the project's dictionaries, or one dictionary's
entries by name. `--json` wraps the list in `dicts`, each with its `case` and
`count`, under `$schema`. One named dictionary prints flat under `$schema`.
That's the list a rule reading the dictionary accepts. To declare a
dictionary, or to see how its entries merge, read
#link(u("/named-sets/#dictionaries"))[Views and dictionaries].

= Templates

`mt explain templates` lists every template `mt add` accepts, one line each:
the template's name, the `kind` it sets, how many slots it declares, and
`default` on the one `mt add` picks when you name none. The `kind` column reads
`any` for a template that leaves the field to a slot, as the built-in `task`
does. The listing has no status column, because `mt add` stamps the status
itself.

The listing reads the same template files `mt add` reads, so a template added
to a directory in `[templates] dirs` appears without a config change. `--json`
wraps the list in `templates` under `$schema`, each with its `name`, `kind`,
`slots` and `default`. To see what one template's slots are, run
`mt add -t <name> --describe`.
