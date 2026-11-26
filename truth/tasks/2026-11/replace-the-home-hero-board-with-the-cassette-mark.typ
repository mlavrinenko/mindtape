#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Replace the home hero board with the cassette mark",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("www", "docs", "stateless", "no-repro"),
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
      docs-match-code: helps[`DESIGN.md` and the hero CSS change with the page markup],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 22),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ 🔴 `www/utils/ui.typ` splits `.hero` into a copy column and a `.hero-board` column. The copy stops near half the main column while a live `mt ls` board fills the right side.
+ 🔴 Below 76rem the board drops under the copy. The first screen ends on a listing instead of the product name.


== TO-BE

- 🟢 The hero is one full-width copy column: kicker, display title, lede, pitch, calls to action and install line.
- 🟢 The cassette mark sits at the right edge of the title row. Its square box matches the display title (`clamp(72px, 7.6vw, 107px)`) and uses the `--green` tint, which lifts to the mint accent in dark mode.
- 🟢 Below 40rem the mark hides. The display title keeps the full width on a phone.
- 🟢 No board markup, board capture or board CSS remains. `hero-ls.json` stays because the proof terminal still reads it.

```html
<section class="hero">
  <div class="hero-copy">
    <p class="kicker">…</p>
    <div class="hero-title">
      <h1>Mindtape</h1>
      <span class="hero-mark" aria-hidden="true"></span>
    </div>
    …
```


== Source of truth

- `www/utils/ui.typ`: `_hero-copy` builds the title row, and `_html-hero` calls it alone.
- `www/assets/styles/components.css`: `.hero`, `.hero-title` and `.hero-mark`.
- `www/assets/images/logo.svg`: the only copy of the mark, masked and tinted per theme.
- `www/generate-reference.sh`: the `home-board` capture the board read.


== Changes

- `www/utils/ui.typ`: remove `_board` and its parsing helpers. Wrap the h1 and the mark in `.hero-title`.
- `www/assets/styles/components.css`: make `.hero` one column, add `.hero-title` and `.hero-mark`, and delete every `.hero-board` and `.board*` rule.
- `www/assets/styles/main.css`: delete the board row transition.
- `www/generate-reference.sh`: delete the `home-board` capture and its JSON slice.
- `DESIGN.md`: replace the departure board with the hero mark in the components, layout and depth rules.


== Out of scope

- The proof section's task file, terminal listing and typeset card. They still show a real task and a real `mt ls` run.
- The Markdown copy (`_md-hero`), which never rendered the board.
- A new illustration or logo variant. The mask reuses `logo.svg`.


== Dogfood

This repo's docs site builds from `www/`. The home page and `DESIGN.md` show the new hero under `just docs::check`.

