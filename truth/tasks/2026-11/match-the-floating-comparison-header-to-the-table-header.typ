#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Match the floating comparison header to the table header",
  kind: "fix",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 10,
  ),
  difficulty: 1,
  tags: ("www", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[a visitor scrolling the comparison table reads the floating header exactly as the table's own],
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
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 25),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/comparison/` in a 1000px-wide window.
+ Scroll until the masthead covers the matrix header and the floating copy appears below it.
+ 🔴 The floating header renders at 12.48px in its normal casing with underlined links on `--paper-2`.
+ 🔴 The table's own header renders at 11px, weight 800, uppercase, tracked, on `--paper` with no underline.

```js
> [".matrix-float-head th", ".matrix thead th"].map(s => {
    const c = getComputedStyle(document.querySelector(s));
    return [s, c.fontSize, c.fontWeight, c.letterSpacing, c.textTransform, c.backgroundColor];
  })
[["…", "12.48px", "700", "0.2496px", "none", "rgb(237, 241, 235)"],
 ["…", "11px", "800", "1.43px", "uppercase", "rgb(245, 247, 242)"]]
```


== Environment

- `docs-redesign` at 120257d1, built with `just docs::build`, on Linux.
- Chromium through `agent-browser` at a 1000×760 viewport.
- `mt 0.6.0`.


== TO-BE

- 🟢 The floating copy of the header renders with the same font size, weight, tracking, casing, colour and background as the table's own header, in both themes.
- 🟢 Its links carry the table header's weight and colour with no underline.
- 🟢 Scrolling the table sideways still moves the copied columns with it and keeps the first cell fixed.

```css
.matrix-float-head th {
  background: var(--paper);
  font-size: 11px; font-weight: 800;
  letter-spacing: 0.13em; text-transform: uppercase;
  color: var(--muted);
}
.matrix-float-head th a { font-weight: 600; letter-spacing: -0.01em; text-decoration: none; }
```


== Changes

- `www/assets/styles/components.css`: match `.matrix-float-head th` to `main thead th`'s font size, weight, tracking, casing, colour and `var(--paper)` background, and match its links to `main table a` with no underline.


== Out of scope

- The clone's script and its horizontal scroll sync.
- The first sticky cell's shadow inside the clone, a separate defect from the type and colour mismatch.
- The table's row content and capability colours.


== Dogfood

This repository's docs site under `www/` is the product's shop window, and the comparison matrix is one of its most-read pages. `just docs::check` builds every page, and the clone appears at any width narrower than the table.

