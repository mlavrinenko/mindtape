#import "../../utils/cli-ref.typ": cli-command, cli-page

#show: cli-page.with("config")

`mt config show` prints the merged config as TOML that `mt` can read back. Add
`--origin` to see which file each value came from.

#cli-command("config")
