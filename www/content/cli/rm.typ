#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("rm")

`mt rm` retires tasks. In a git repository it records a `swh:1:rev` id for
each task, rewrites every `links:` edge to that id, and deletes the file. It
doesn't commit. Outside git, #link(u("/config/"))[`[rm] strategy`] decides
what happens to those edges.

Refer to a task by stem, hash or path. Give a folder under a task root instead,
and `mt rm` retires every task inside it too, as if you had listed each file.
Try `mt rm tasks/sw1 --dry-run` to see the list first. `mt rm` refuses a folder
outside every root.

A #link(u("/hooks/#removal-hooks"))[`[[on.rm]]`] hook runs before the removal
writes anything. A gate then refuses it while every file is still there. A dry
run doesn't run a hook.

#cli-command("rm")
