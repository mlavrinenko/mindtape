#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("tags")

`mt tags` lists every tag `[tasks.tags] allowed` permits, with the rules that
read it. A rule reads a tag when its `when` or `assert` names
`has_tag("<tag>")`, so a new rule needs no registration. Each rule shows the
message a breach prints, the same line `mt rules` gives it.

Without `--for`, a tag that no rule reads is still a legal tag, so the listing
shows it with `area label` rather than dropping it. That covers tags a view or
a person reads, such as the area labels a project keeps for grouping.

`--for` keeps only the tags one step reads, by the same moments
#link(u("/cli/rules/"))[`mt rules --for`] uses. `add` keeps the tags read by
the rules `mt add` checks before it writes, `flip` adds the flip actions, and
`commit` keeps the commit rules. `--json` returns a `tags` map, each kept tag
to its rules and their messages, under `schema_version`.

#cli-command("tags")
