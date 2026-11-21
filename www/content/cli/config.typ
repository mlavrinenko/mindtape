#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../generated/just.typ" as just

#show: cli-page.with("config")

`mt config show` prints the merged config as TOML that `mt` can read back:
every key a layer set, from the embedded floor through the project's
`.mindtape/config.toml`. Add `--origin` to see which file each value came from,
and `--json` to print the same table under `config`, with `$schema` first.

The collected table carries the keys a layer set, so copying it into a config
pins only what the project chose. `--resolved` prints what `mt` uses instead,
every default filled in. A rule that sets no `severity` shows the level it
reports at, and a section a layer leaves out prints the keys `mt` falls back
to. Written back as a config, the resolved output resolves to itself. It leaves out
`extends`, `disable` and `[pack-sources]`, because those say how the layers
combine rather than what the config does. `--defaults` prints just the embedded
floor, with no project needed. It and `--origin` answer other questions, so
neither combines with `--resolved`.

= The essentials report

The report at `contrib/jq/reports/essentials.jq` prints a project's shape from
the resolved config. The output covers the roots and their exclusions, the
status legend, the actors and roles, and the priority framework with its bounds
and inputs. It covers difficulty, estimate, kinds, link kinds, comments,
extensions and templates. It covers the commit trailer and run, the hooks, the
`[rm]` filter, the tags, the views, the dictionaries and the derived fields.
Every rule appears with the moments it fires at: `add`, `flip`, `commit` or
`rm`. A section the project leaves empty prints empty. A reader can then tell
"no hooks" from a report that forgets to print hooks.

`mt config show --json --resolved | jq -f contrib/jq/reports/essentials.jq`
feeds it. The report reads the config `mt` uses, not a second shape of it. The
report's top-level keys are the config's own, less the keys its `dropped` array
names. `jq -n -f contrib/jq/reports/essentials.jq` prints that array alone.

#just.meta-jq-reports() runs every report in `contrib/jq/reports/` against
`contrib/jq/fixtures/config.resolved.json` and diffs the output with the
report's expected file. It also fails when the config gains a top-level key
that a report neither prints nor drops. A report cannot fall behind in
silence.

The report is this repo's own until it settles. It retires when
`git log --since="1 month ago" -- contrib/jq/reports/` is empty at a release
review, which means no one has had to correct it for a month. To keep a report
of its own, a project copies the file into its tree under
`contrib/jq/reports/` and edits the copy. The reports stay out of the config
package until then.

#cli-command("config")
