#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("rm")

`mt rm` retires tasks. In a git repository it records a `swh:1:rev` id for
each task, points every `links:` edge at it, and deletes the file, without
committing. Outside git, #link(u("/config/"))[`[rm] strategy`] decides what
happens to those edges.

#cli-command("rm")
