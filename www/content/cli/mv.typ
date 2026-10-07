#import "../../utils/cli-ref.typ": cli-command, cli-page

#show: cli-page.with("mv")

A task's id is its file stem, so renaming the file renames the task. Rename it
by hand and every `links:` edge that points at it breaks. `mt mv` renames it
and rewrites those edges in the same checked write, so `mt check` finds no
broken edge afterwards. To change the `title:` field, use `mt set`.

#cli-command("mv")
