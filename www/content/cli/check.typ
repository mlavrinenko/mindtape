#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("check")

`mt check` checks every task: field values, required fields, numeric bounds,
links and dependency cycles. It also checks the task each template files with
its defaults, as #link(u("/templates/"))[Templates & slots] describes. It
prints each problem it finds and exits non-zero if there's at least one. Run
it after you edit a task file by hand.
#link(u("/concepts/#where-a-mistake-is-caught"))[Concepts] says which mistakes
the prelude catches and which `mt check` catches, and
#link(u("/rules/"))[Rules] covers the checks you can add. To check a commit
message instead, run `mt check --commit-msg <file>` from a `commit-msg` hook.
It checks the message against the project's
#link(u("/commit-rules/"))[commit rules]. In CI, `mt check --commits <range>`
checks every commit in a range.

#cli-command("check")
