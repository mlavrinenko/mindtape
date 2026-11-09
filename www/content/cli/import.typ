#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("import")

`mt import` writes each record through the same destination guard as `mt add`,
so it refuses a template directory, a reserved `.mindtape` or `.git` path, a
destination that is itself a symlink, and a new file under a Git ignore rule.
#link(u("/concepts/#which-files-count-as-tasks"))[Which files count as tasks]
defines the paths that qualify.

#cli-command("import")
