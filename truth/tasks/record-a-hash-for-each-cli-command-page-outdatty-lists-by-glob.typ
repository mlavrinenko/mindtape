#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Record a hash for each CLI command page outdatty lists by glob",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "tooling"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[an edit to a command page leaves a review record like every other dependent],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Integrating the per-command CLI pages and the docs-terms pass, which edited those pages.
+ 🔴 #raw("outdatty.yaml"): The \`cli-docs\` group lists \`www/content/cli/\*.typ\` as a glob dependent, and \`outdatty.lock\` records no hash for any file it matches, so a reviewed edit to one command page leaves no record.

`outdatty.yaml:150`: `- www/content/cli/*.typ`
`rg -c 'content/cli/' outdatty.lock` prints nothing, while `www/content/cli.typ` has its hash at lines 73 and 103.
Appending a line to `www/content/cli/add.typ` and running `outdatty check` reports nothing.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("outdatty.yaml").

== TO-BE

- 🟢 Each command page is a dependent outdatty records, so `just outdatty-update cli-docs <path>` after an edit to one leaves a hash for it.
- 🟢 A new command page joins the group without a hand edit, or a gate names the page missing from it.


== Changes

- `outdatty.yaml`: list the command pages so outdatty records them, or confirm with outdatty's docs that a glob dependent records per-file hashes and drop this task.
- `outdatty.lock`: the recorded hashes.


== Rollback

Revert the commit; the glob stays as before.

== Out of scope

- Other groups' dependents.


== Dogfood

This repo's next edit to a command page records its review in `outdatty.lock`.

