#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("recur")

`mt recur` is the only command that advances a `recur:` field. Read commands
such as `mt ls` never change it. #link(u("/recurring/"))[Recurring tasks]
describes the schedule format and what a new occurrence keeps.

#cli-command("recur")
