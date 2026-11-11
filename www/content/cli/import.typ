#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("import")

`mt import` writes each record through the same destination guard as `mt add`,
so it refuses a template directory, a reserved `.mindtape` or `.git` path, a
destination that is itself a symlink, and a new file under a Git ignore rule.
#link(u("/concepts/#which-files-count-as-tasks"))[Which files count as tasks]
defines the paths that qualify.

`--dir <path>` sends the whole batch there: a relative path resolves from the
project root, and an absolute one is taken as-is. Without it, each record keeps
the folder its `file` field names. The removed `--into` and `--to` are refused
with a hint to `--dir`.

#cli-command("import")
