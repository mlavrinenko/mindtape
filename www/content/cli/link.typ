#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("link")

`mt link` adds edges to a task's `links:` field. You name each target by its
stem, and `mt` writes the path to its file for you. It's the only command
that writes `links:`. #link(u("/links/"))[Task links] describes the edge
kinds, how `mt` checks a target, and the map `--apply` reads.

#cli-command("link")
