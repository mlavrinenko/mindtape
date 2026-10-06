#import "../../utils/cli-ref.typ": cli-command, cli-page

#show: cli-page.with("config")

`mt config show` prints the merged config as TOML that `mt` can read back,
and `--origin` names the file each value came from.

#cli-command("config")
