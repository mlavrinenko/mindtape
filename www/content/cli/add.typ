#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("add")

`mt add` writes a task source, so it refuses a destination the loader would
not read back: a template directory, a reserved `.mindtape` or `.git` path, a
destination that is itself a symlink, and a new file under a Git ignore rule.
It leaves no file behind.
#link(u("/concepts/#which-files-count-as-tasks"))[Which files count as tasks]
defines the paths that qualify.

#cli-command("add")
