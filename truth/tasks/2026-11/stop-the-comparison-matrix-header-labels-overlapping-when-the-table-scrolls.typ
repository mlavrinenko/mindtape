#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Stop the comparison matrix header labels overlapping when the table scrolls",
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
      readable-tasks: helps[a visitor scanning the comparison table reads each column header without labels drawing on top of each other],
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
  status: done(2026, 11, 25),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/comparison/` in a 1000px-wide window.
+ Scroll to "Feature by feature" so its header clears the masthead.
+ Scroll the table right by about 100px.
+ 🔴 The sticky `Feature` header cell has no background, so the `Mindtape` header slides underneath it and both labels draw in the same place. The body's fixed column paints `var(--paper)` and hides the cells behind it. The header does not.

```
> getComputedStyle(document.querySelector(".matrix thead th:first-child")).backgroundColor
"rgba(0, 0, 0, 0)"
```


== Environment

- `docs-redesign` at d9b27487, built with `just docs::build`, on Linux.
- Chromium through `agent-browser` at a 1000×760 viewport.
- `mt 0.6.0`.


== TO-BE

- 🟢 The `Feature` header cell paints the same `var(--paper)` as the fixed body column, so the headers that scroll behind it stay hidden.
- 🟢 The header column keeps its shadow and stays legible in both themes.

```css
.matrix thead th:first-child {
  position: sticky; left: 0; z-index: 2;
  background: var(--paper);
}
```


== Changes

- `www/assets/styles/components.css`: add `background: var(--paper)` to `.matrix thead th:first-child`, beside the `sticky`, `left` and `z-index` that rule already sets.


== Out of scope

- The floating header clone, whose first column scrolls away with the rest. That is a separate defect from the labels drawing on top of each other.
- The comparison copy and the capability colours.


== Dogfood

This repository's docs site under `www/` is the product's shop window, and the comparison table is one of its most-read pages. `just docs::check` builds every page, and the header scrolls at any width narrower than the table.
