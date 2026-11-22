#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Fix the product name in the site chrome",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("www", "docs", "stateless"),
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
      docs-match-code: helps[the site chrome prints the same product name as `PRODUCT.md`],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 22),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ 🔴 The section strip under the masthead ends with `MIND TAPE / LOCAL`, a label that marks no place and splits the product name into two words.
+ 🔴 The footer repeats the split name as `MIND TAPE · MIT · 2026`.


== TO-BE

- 🟢 The section strip ends at its last station, `Integrate`, with no trailing label.
- 🟢 The footer prints `MINDTAPE · MIT · 2026`, matching the product name.


== Source of truth

- `www/templates/layout.typ`: the `_route-strip` section nav and the name span in the footer.
- `www/assets/styles/main.css`: the `.line-end` rule the strip's label used.


== Changes

- `www/templates/layout.typ`: delete the `line-end` span from `_route-strip`, and print `MINDTAPE` in the footer.
- `www/assets/styles/main.css`: delete the `.line-end` rule and its narrow-screen override.


== Out of scope

- The station names and their all-caps style. They name sections, not the product.
- The rail footer, which already prints the version and `MIT`.


== Dogfood

This repository's docs site is the first reader: `just docs::check` builds every page, and its header and footer carry the corrected name.

