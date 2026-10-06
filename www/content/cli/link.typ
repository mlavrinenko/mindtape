#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("link")

`mt link` is the only command that writes `links:`.
#link(u("/links/"))[Task links] describes the edge kinds, how `mt` checks a
target, and the map `--apply` reads.

#cli-command("link")
