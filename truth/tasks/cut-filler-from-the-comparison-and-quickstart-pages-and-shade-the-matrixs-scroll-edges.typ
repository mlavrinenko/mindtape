#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Cut filler from the comparison and quickstart pages and shade the matrix's scroll edges",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader of the comparison reads only sentences that tell them something],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
)

== Problem

+ Open the comparison page.
+ 🔴 It says twice that it compares local trackers, explains why hosted trackers are absent, and says the table scrolls sideways.
+ Open the quickstart and read "Change status".
+ 🔴 "`flow` allows `proposed` to `wip` and `wip` to `done`" reads like a fixed workflow.


== TO-BE

- 🟢 The comparison opens with one lede and goes straight to the tools.
- 🟢 The matrix shows a shadow on each edge it can scroll past, and no sentence explains the scrolling.
- 🟢 The quickstart says the statuses and moves are the project's own config.


== Source of truth

- The owner's docs review of the comparison and quickstart pages.


== Changes

- `www/content/comparison.typ`: the lede, the cut sentences, the browser-board item.
- `www/templates/layout.typ`: the matrix script marks which edges can scroll.
- `www/assets/styles/components.css`: a shadow on each such edge.
- `www/content/quickstart.typ`: the status paragraph and the `mt check` sentence.


== Out of scope

- The todo.txt and maturity items, which wait on the owner's call.


== Dogfood

Not applicable: the change is two docs pages this repo doesn't read to work.

