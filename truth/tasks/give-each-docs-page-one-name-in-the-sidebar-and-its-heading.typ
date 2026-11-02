#import "@local/mindtape:0.3.0": *
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
    product: verdicts(
      (
        readable-tasks: helps[a page reads the same name in the sidebar and its heading],
      ),
      "implementable-tasks",
      "enforced-practice",
      "cheap-adoption",
      "typst-over-markdown",
      "plain-files",
    ),
    process: verdicts(
      (:),
      "sound-codebase",
      "docs-match-code",
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    12,
  )[seven pages carry one name in the sidebar and their heading],
)

== AS-IS

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
