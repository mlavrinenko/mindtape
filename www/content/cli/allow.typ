#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("allow")

A config can name commands for `mt` to run. A
#link(u("/wait-on-upstream/"))[`[[tasks.waits]]`] checker runs while `mt`
reads, a #link(u("/hooks/"))[`[[on.flip]]`] `run` action runs when
`mt flip` changes a status, and the `[go] editor` and `[add] editor` programs
open when you ask for an editor. `mt` runs none of them in a repo until you run
`mt allow` there. Read them first. They run with your rights. An editor in your
own `mindtape/defaults.toml` is yours already, so it needs no approval.

#cli-command("allow")
