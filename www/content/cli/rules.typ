#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("rules")

`mt rules` lists every rule your project checks, one line each. A line holds
the rule's severity, its name as `mt config show --origin` prints it, and the
message a breach prints. It lists what's left after packs, overrides and
`disable`, so a project that extends a #link(u("/packs/"))[rule pack]
sees the rules it actually got. A flip action that carries a `message` shows
up too, named by its place in `[[on.flip]]`.

A rule with no `severity` shows the level `mt` applies to it. That's `warn`
for `[[tasks.rules]]` and `[[commits.rules]]`, and `error` for a body rule. A
rule with no `message` shows the text a breach prints instead, or who words
it.

Under the rules, `mt rules` lists every derived field a
#link(u("/config/#derived-fields"))[collector] computes: its type, its
`derived.<name>`, its kind and origin, and the rules that read it. A Typst
collector names its module and function; an ast-grep collector names its
language, pattern and optional glob. `--for commit` omits that block, since a
commit rule cannot read a derived field.

`--for` keeps the rules one step checks. `add` lists what `mt add` checks
before it writes, `flip` adds the flip actions to that, and `commit` lists
your #link(u("/commit-rules/"))[commit rules]. `--json` adds each rule's
section, `when` and `assert`.

#cli-command("rules")
