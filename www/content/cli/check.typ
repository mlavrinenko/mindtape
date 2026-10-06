#import "../../utils/cli-ref.typ": cli-command, cli-page
#import "../../utils/site.typ": u

#show: cli-page.with("check")

`mt check` validates every task: field values, required fields, numeric
bounds, links and dependency cycles. It also checks the task each template
files with its defaults, as #link(u("/templates/"))[Templates & slots]
describes. It prints each problem and exits non-zero when there is one.
#link(u("/concepts/"))[Concepts] describes the two layers that check a task,
and #link(u("/rules/"))[Rules] the checks a project adds.
`mt check --commit-msg <file>` checks a commit message instead, from a
`commit-msg` hook, against the project's
#link(u("/commit-rules/"))[commit rules]. `mt check --commits <range>` checks
every commit in a range, for CI.

#cli-command("check")
