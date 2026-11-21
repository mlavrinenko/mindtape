#import "../../utils/cli-ref.typ": cli-command, cli-page

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

#cli-command("config")
