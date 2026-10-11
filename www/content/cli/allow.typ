#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("allow")

A config can name commands that `mt` runs while it reads, such as a
#link(u("/wait-on-upstream/"))[`[[tasks.waits]]`] checker. `mt` runs none of
them in a repo until you run `mt allow` there. Read them first. They run with
your rights.

#cli-command("allow")
