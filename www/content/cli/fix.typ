#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("fix")

`mt fix` rewrites the files an `mt` upgrade left behind. One fixer,
`prelude-import`, changes an import of `@local/mindtape` at a version this
binary does not read to the current version. It parses the Typst, changes
only the version string, and leaves every other byte alone. `mt fix --list`
names the fixers, and `mt fix prelude-import` runs one.

Without a flag, it scans every
#link(u("/concepts/#which-files-count-as-tasks"))[task source] and every
project template dir. `--all` scans every Git-visible `.typ` file in the
project instead, non-task files such as `lib/` and docs included. Outside a
Git work tree, `--all` walks the project with the `.git`, `target` and
`.mindtape/cache` skips. Neither scope enters a symlinked directory, an
untracked ignored file is in neither, and a tracked task under an ignore rule
is in both. `--dry-run` prints each edit and writes nothing. `--check` prints
one diagnostic per unsupported version, names the first file that uses it and
the versions this `mt` supports, and exits 1 while one remains, so CI can gate
it with `mt fix --all --check`.

An import of a supported older version, such as `0.2.0`, is kept and reported.
A newer version is refused with an upgrade hint. A malformed spec, an
unreadable file or a symlinked file with an edit is reported, and any of them
stops the run before a file changes. A project-local
`[[typst.local-packages]]` pin is never edited. When it declares the imported
unsupported version, the import is reported unfixable. A successful write ends
with `mt check`.

#cli-command("fix")
