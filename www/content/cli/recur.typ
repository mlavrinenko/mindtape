#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("recur")

`mt recur` is the only command that advances a `recur:` field. Read commands
such as `mt ls` never change it. #link(u("/recurring/"))[Recurring tasks]
describes the schedule format and what a new occurrence keeps.

`mt recur` writes a new occurrence through the same destination guard, so it
refuses a template directory, a reserved `.mindtape` or `.git` path, a
destination that is itself a symlink, and a new file under a Git ignore rule,
and suggests moving the task or unignoring its path.
#link(u("/concepts/#which-files-count-as-tasks"))[Which files count as tasks]
defines the paths that qualify.

#cli-command("recur")
