#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("add")

`mt add` writes a task source, so it refuses a destination the loader would
not read back: a template directory, a reserved `.mindtape` or `.git` path, a
destination that is itself a symlink, and a new file under a Git ignore rule.
It leaves no file behind.
#link(u("/concepts/#which-files-count-as-tasks"))[Which files count as tasks]
defines the paths that qualify.

`--set` names a template slot or a core field `mt set` writes. A field the
template has no slot for is inserted after fill, so
`mt add --set due=2026-12-01` works on any template. `--dir <path>` names one
destination: a relative path resolves from the project root, and an absolute
one is taken as-is. The removed `--into` and `--to` are refused with that hint.

`--priority`, `--difficulty`, `--kind`, `--assignee`, `--due` and `--start`
are direct aliases for `--set` on those scalar fields. `mt add` runs the
project's #link(u("/hooks/#add-hooks"))[`[[on.add]]`] hooks after the values
are collected, so a hook can fill a field or choose the destination. An
explicit `=` value (a `--set`, a positional title or an `--edit` change) or an
explicit `--dir` wins; an explicit `+=` appends and does not block a hook
`set`, and `--no-hooks` skips the hook list.

#cli-command("add")
