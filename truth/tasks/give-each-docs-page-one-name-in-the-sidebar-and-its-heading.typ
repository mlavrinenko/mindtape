#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Give each docs page one name in the sidebar and its heading",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 1,
  tags: ("www", "docs"),
  estimate: (src-files: 9),
  x-goals: (
    product: (
      readable-tasks: helps[a page reads the same name in the sidebar and its heading],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
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

+ Click "Statuses" in the docs sidebar.
+ 🔴 The page's heading says "The status legend", and seven more pages carry two names.


== TO-BE

- 🟢 Each page's title equals its sidebar label, and its summary carries the detail.


== Source of truth

- The page manifest each `www/content/*.typ` declares in its `meta`.


== Changes

- `www/content/*.typ`: one name per page, and the links that spell the old title.


== Out of scope

- Routes, which stay as they are.


== Dogfood

Not applicable, this repo's docs readers are its users.
