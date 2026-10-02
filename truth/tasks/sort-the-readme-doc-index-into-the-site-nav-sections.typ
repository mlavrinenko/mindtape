#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Sort the README doc index into the site NAV sections",
  kind: "dev",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (src-files: 2),
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
      docs-match-code: helps[README's index follows the site's structure with no hand edits],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
)

== Problem

+ Read the end of `README.md`, "The docs site has the rest".
+ 🔴 It is one flat list of 22 pages, each with its full summary: about half the file.
+ The site already groups the same pages into Start, Model, Tasks, Reference and Integrate (`NAV` in `www/utils/site.typ`). `www/generated/site-pages.json` does not carry the groups, so `docs/src/readme.typ` cannot read them.


== TO-BE

- 🟢 `site-pages.json` carries NAV's groups in order.
- 🟢 README shows one line per group: the group name, then the page titles as links. No summaries.
- 🟢 A page added to NAV, or moved between groups, appears in README with no README edit.


== Changes

- `www/generate-site-pages.sh`: emit `groups: [{title, routes}]` from NAV. First make `readme.typ` read `groups` and see the render fail on today's manifest.
- `docs/src/lib.typ`: a reader for the groups.
- `docs/src/readme.typ`: the grouped index.


== Rollback

Revert the commit; README goes back to the flat list.


== Out of scope

- The rest of README: the README task rewrites it.


== Dogfood

This repo's README renders the grouped index on every `just docs md`.

