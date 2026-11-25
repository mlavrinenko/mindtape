#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Move the docs Copy as Markdown button into the title row",
  kind: "docs",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("www", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[`DESIGN.md` records the button in the title row and the lede only when a page has a summary],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 25),
)

== AS-IS

+ Build the docs site and open `/cli/add/` at 1280px.
+ 🔴 The title row holds only the `h1`. The Copy as Markdown button sits in an empty `.article-lede` row below it, because that command page has no summary.


== TO-BE

- 🟢 Every article page renders the Copy as Markdown button in the title row, aligned with the `h1`'s right edge.
- 🟢 A page with a summary renders its lede under the title row, and a page without one renders no empty row.
- 🟢 The button keeps its tooltip, its Copied state and its keyboard focus, and at 390px it stays inside the viewport without overlapping the title.


== Source of truth

- `www/templates/layout.typ`: where the article head renders the `h1`, the summary and the button.
- `www/assets/styles/main.css`: the `.article-head` rules.


== Changes

- `www/templates/layout.typ`: wrap the `h1` and the copy button in a title row, and render the lede only when the page has a `summary`.
- `www/assets/styles/main.css`: style the title row, move the title's bottom margin to the lede, and update the narrow-width rules.
- `DESIGN.md`: record the title-row button and the summary-only lede.


== Out of scope

- The button's copy script, its tooltip text and the Markdown copies themselves.
- The home page, which renders its own hero and no copy button.


== Dogfood

The docs site itself: `/estimates/` shows a page with a lede and `/cli/add/` one without, checked at 1280px and 390px.

