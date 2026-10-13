#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("rm")

`mt rm` retires tasks. In a git repository it records a `swh:1:rev` id for
each task, points every `links:` edge at that id, and deletes the file. It
doesn't commit. Outside git, #link(u("/config/"))[`[rm] strategy`] decides
what happens to those edges.

Name a task by stem, hash or path, or name a folder under a task root and
every task inside it goes too, as if you had listed each file. Try
`mt rm tasks/sw1 --dry-run` to see the list first. A folder outside every
root is refused.

#cli-command("rm")
