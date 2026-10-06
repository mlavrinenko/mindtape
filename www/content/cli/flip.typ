#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("flip")

`mt flip <status> <refs…>` stamps a status dated today on one or more tasks.
Each status in the #link(u("/statuses/"))[status legend], the project's list
of allowed statuses, is also a command, so `mt done x` is `mt flip done x`.
The legend sets which changes are allowed and which need a note. `mt flip` is the only command that
runs #link(u("/hooks/"))[`[[on.flip]]` hooks].

#cli-command("flip")
