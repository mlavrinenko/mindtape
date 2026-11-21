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
`.mindtape/cache` skips. Both scopes skip a symlinked directory and leave out
an untracked ignored file. Both include a tracked task under an ignore rule.
`--dry-run` prints each edit and doesn't write anything.

A run prints one row per import it finds, `rewrite` for an import it changes
and `kept` for a supported older version it leaves alone. It ends on a line
with both counts. `--check` doesn't write. It prints the rewrites and the
summary. It reports each unsupported version with the first file that uses
it and the versions this `mt` supports, and exits 1 while one remains. CI gates
that with `mt fix --all --check`. `-v` adds the `kept` rows to the report, the
scan record a reader gets as one line per import. `-q` doesn't print a row or
the summary while a run succeeds, in either mode. Every diagnostic still
prints. A gate reads one line per problem, and a clean tree leaves the output
empty: `mt fix --all --check -q`.

`mt fix` keeps and reports an import of a supported older version, such as
`0.2.0`. It refuses a newer version with an upgrade hint. It reports a
malformed spec, an unreadable file or a symlinked file with an edit.
Any of them stops the run before a file changes. `mt fix` never edits a
project-local `[[typst.local-packages]]` pin. When it declares the imported
unsupported version, `mt fix` reports the import as unfixable. A successful
write ends with `mt check`.

#cli-command("fix")
