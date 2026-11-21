#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("allow")

A config can name commands for `mt` to run. A
#link(u("/wait-on-upstream/"))[`[[tasks.waits]]`] checker runs while `mt`
reads, a #link(u("/hooks/"))[`[[on.flip]]` or `[[on.rm]]`] `run` action runs
when `mt flip` changes a status or `mt rm` removes a task, and the `[go] editor`
and `[add] editor` programs open when you ask for an editor. `mt` runs none of them in a repo until you run
`mt allow` there. Read them first. They run with your rights. An editor declared in your
own `mindtape/defaults.toml` doesn't need approval.

`mt allow` prints every command under one line saying whether the repository
allows them, then records the approval. `mt allow --status` prints the same
table and doesn't record anything, so you can read what the config runs before
you trust it. `--json` prints that list for a script: each site's `kind`
(`gate` or `editor`), its `moment`, the hook's `label` when it has one, and its
`command`.

#cli-command("allow")
