#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Turn the typeset footnote into an expandable caption",
  kind: "docs",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("www", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: helps[the home page shows the same Typst task as source and as a typeset page],
      plain-files: neutral,
      self-describing: helps[the action says the page opens in place, not on a separate route],
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 24),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ Scroll to the task under "Here's a real task." and find the block at the bottom of its code panel.
+ 🔴 The block is an uppercase 11px mono footer reading `THE SAME FILE ALSO TYPESETS AS A DOCUMENT` with `Take a look →` at the far right. It reads as terminal status information, not text, and the arrow suggests a link to another page.


== TO-BE

- 🟢 The block is a serif caption under the panel: `The file above also typesets as a document. Show the page`, with a down chevron.
- 🟢 The action becomes `Hide the page` with an up chevron while the preview is open.
- 🟢 A click anywhere on the caption opens the typeset page in place below, at the 690px reading-column width, rather than navigating away.
- 🟢 The caption has a hover state and a 2px focus ring, and keeps the code text's 24px left alignment so it stays tied to the file above.


== Source of truth

- `www/utils/ui.typ`: `_html-hero()` writes the `<details>` and its summary.
- `www/assets/styles/components.css`: `.proof-typeset`, `.proof-foot`, `.proof-open` and `.typeset-card` style the caption, chevron and preview.
- `www/generated/hero-card.svg`: the page image the details reveals.


== Changes

- `www/utils/ui.typ`: replace the uppercase footer and arrow with a sentence and a Show/Hide action for the page.
- `www/assets/styles/components.css`: set the caption in the reading voice, draw the state chevron, add hover and focus states, and open the preview at the reading-column width.


== Out of scope

- The code panel and the `mt ls` terminal. They already show the source and the listing.
- The page image itself. `www/generate.sh` keeps producing `hero-card.svg`.
- The Markdown copy. It keeps its sentence and cannot carry the inline SVG preview.


== Dogfood

`just docs::check` builds and validates the site root, and `just docs::serve` shows the caption open and closed at `/mindtape/`.

