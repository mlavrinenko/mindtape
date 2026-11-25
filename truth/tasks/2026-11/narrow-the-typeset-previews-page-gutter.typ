#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Narrow the typeset preview's page gutter",
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
      typst-over-markdown: helps[the home page's typeset face shows the task as a document, not a small page lost in a large frame],
      plain-files: neutral,
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 25),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ Expand the caption under the task file: "The file above also typesets as a document. Show the page".
+ 🔴 The page sits in a card with about 80px of white between its border and the content. The compiled page adds a 1.6cm margin on every side. The card adds 16px of its own padding. The card also stretches the page across the full reading column. A small document floats in a large empty frame.

== TO-BE

- 🟢 The preview crops a little over half of the compiled page's margin. It carries no padding of its own. The content starts about 25px inside the border.
- 🟢 The card is 30rem wide. The page keeps its compact type size instead of scaling to the width of the code panel above it.

== Source of truth

- `www/crop-page-margin.py`: reads the page margin from the compiled SVG's first `typst-group` transform. It rewrites the root `viewBox`, `width` and `height`.
- `www/generate.sh`: compiles `hero-card.svg` and runs the crop on it.
- `www/assets/styles/components.css`: `.typeset-card` sets `max-width: 30rem` and `padding: 0`.

== Changes

- `www/crop-page-margin.py`, new: crop 55% of the page margin from every side. Fail the build on an SVG that is not the expected one-page `typst-doc`.
- `www/generate.sh`: run the crop after `typst compile` writes `hero-card.svg`.
- `www/assets/styles/components.css`: drop the redundant 16px padding and narrow the card to 30rem.

== Out of scope

- The `task` render in `lib/prelude.typ`. Its page margins stay as they are. Cropping the page is a docs preview choice.
- Other generated captures and pages. Only the hero card changes.

== Dogfood

The home page's "Here's a real task." preview renders the task at 30rem with the cropped page margin. `just docs::check` builds it.
