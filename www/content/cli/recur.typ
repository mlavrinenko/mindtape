#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("recur")

Only `mt recur` advances a `recur:` field. No read command changes it.
#link(u("/recurring/"))[Recurring tasks] describes the schedule format and
what a new occurrence keeps.

#cli-command("recur")
