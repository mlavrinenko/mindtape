#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("allow")

A config can name commands for `mt` to run. A
#link(u("/wait-on-upstream/"))[`[[tasks.waits]]`] checker runs while `mt`
reads, and a #link(u("/hooks/"))[`[[on.flip]]`] `run` action runs when
`mt flip` changes a status. `mt` runs none of them in a repo until you run
`mt allow` there. Read them first. They run with your rights.

#cli-command("allow")
