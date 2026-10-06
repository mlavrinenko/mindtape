#import "../../utils/cli-ref.typ": cli-command, cli-page

#show: cli-page.with("mv")

A task's id is its file stem, so renaming the file renames the task. `mt mv`
renames it and rewrites every `links:` edge that points at it in the same
checked write, so `mt check` finds no broken edge afterwards. The `title:`
field is separate, and `mt set` changes it.

#cli-command("mv")
