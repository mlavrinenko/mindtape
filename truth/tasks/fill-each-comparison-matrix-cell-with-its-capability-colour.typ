#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Fill each comparison matrix cell with its capability colour",
  kind: "dev",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  estimate: (src-files: 1),
  tags: ("found", "www"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader scans the matrix by colour, and a half-filled cell reads as a different rating],
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

+ Checking the scroll shadows on the comparison page at a 900-pixel viewport.

+ 🔴 #raw("www/assets/styles/components.css"): A coloured matrix cell fills only the height of its own text, so in a row with a taller neighbour the tile stops halfway down the cell.


The `.matrix td .cap` rule sets `display: block` and a negative margin, so the span is as tall as its text; in the "Query and JSON" row the green "Filter language; JSON and CSV" tile ends above the bottom of its cell while the amber dstask cell beside it runs four lines longer.


This fell outside the task that surfaced it, so it is filed here rather than
left in a closing report no one rereads. It belongs in
#raw("www/assets/styles/components.css").

== TO-BE

- 🟢 Each rated cell of the comparison matrix is coloured from border to border, however tall its row.


== Changes

- `www/assets/styles/components.css`: colour the `td` that holds a `.cap` span, through `:has()`, instead of the span alone.


== Rollback

- Revert the `components.css` rule.


== Out of scope

- The floating header clone, whose first column scrolls away with the rest: its own finding if it bothers anyone.


== Dogfood

Not applicable: the comparison page is read by visitors, not by this repo's work.

