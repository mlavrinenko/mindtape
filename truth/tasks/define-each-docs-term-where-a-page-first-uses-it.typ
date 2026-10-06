#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Define each docs term where a page first uses it",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader who lands on any page from search follows it without reading the pages before it],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Open `/split-oversized-tasks/`, the first page in the sidebar.
+ 🔴 Its config uses `when = "!terminal"` and the page never says what a terminal status is. Only `/statuses/` defines it.
+ 🔴 Other terms appear before any definition or link: "slot" on the Quickstart, "legend" on the Quickstart, "floor" on Concepts ("sits on top of a floor built into `mt`"), "facets" in the JSON on the agents page.


== TO-BE

- 🟢 On every page, a term the page uses is defined in one clause at its first use, or links to the one page that defines it.
- 🟢 The audit covers at least terminal, slot, legend, floor, layer, facet, stem, view, rule pack, gate and flip, reading the pages in sidebar order.


== Source of truth

- The code that gives each term its meaning, such as `terminal` in the statuses config.
- The page each term is defined on now.


== Changes

- `www/content/*.typ`: the definitions and links where each term first appears.


== Out of scope

- A glossary page. Each term keeps one home page, and the other pages link it.
- A check that enforces definitions.


== Dogfood

Not applicable: the change is to the site's prose.

