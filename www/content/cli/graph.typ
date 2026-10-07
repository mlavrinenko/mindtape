#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("graph")

`mt graph` prints the dependency edges that `ready` and `blocked` read, as
Graphviz DOT or Mermaid. Each node shows a task's title and status, coloured
by whether the task is ready. #link(u("/links/"))[Task links] describes the
edge kinds.

#cli-command("graph")
