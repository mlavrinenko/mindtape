#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Copy a docs page as Markdown from an icon button beside its title",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 3),
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
      docs-match-code: helps[an agent or a person copies the exact Markdown the site renders, in one click],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
)

== Problem

+ Open any docs page.
+ 🔴 The header's "Markdown" link sits next to GitHub, and nothing says it opens the page's Markdown copy.


== TO-BE

- 🟢 A Markdown icon button sits beside each page's title. Hovering it shows "Copy as Markdown", and clicking it copies the page's Markdown.
- 🟢 The header holds no Markdown link. Agents still find each page's Markdown through `llms.txt` and the page's `rel="alternate"` link.


== Source of truth

- `www/templates/layout.typ`: where the header and the page title render.


== Changes

- `www/templates/layout.typ`: the button and its script, the header link gone.
- `www/assets/styles/components.css`: the button and its tooltip.
- `www/assets/images/markdown.svg`: the icon, if no inline one fits.


== Out of scope

- The Markdown copies themselves.


== Dogfood

Not applicable, this repo's agents read the Markdown through `llms.txt`.
