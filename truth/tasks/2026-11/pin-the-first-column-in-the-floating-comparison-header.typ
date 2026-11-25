#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Pin the first column in the floating comparison header",
  kind: "fix",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 2,
  tags: ("www", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[a reader keeps the feature names in view while the pinned header follows the table],
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
+ Scroll until the table top passes under the masthead, then scroll the table right by 200px.
+ 🔴 The floating header's first column scrolls away with the rest. The body's first column stays pinned, so the `Feature` header sits off the left edge and the pinned body column lines up with a different header.

```
> document.querySelector('.matrix-float-head').getBoundingClientRect().left
311
> document.querySelector('.matrix-float-head th').getBoundingClientRect().left
111
```


== Environment

- `docs-redesign` at 7050c11c, built with `just docs::build`, on Linux.
- Chromium through `agent-browser` at a 1000×760 viewport.
- `mt 0.6.0`.


== TO-BE

- 🟢 The floating header pins `Feature` to the table's left edge, in the same column width the body keeps.
- 🟢 An opaque background covers the headers that scroll beneath the pinned cell.
- 🟢 The other headers keep their column positions and slide with the table.

```
> document.querySelector('.matrix-float-head').getBoundingClientRect().left
311
> document.querySelector('.matrix-float-head th').getBoundingClientRect().left
311
```


== Changes

- `www/assets/styles/components.css`: keep the clone's first header cell sticky at `left: 0` with an opaque background, as the live header's first cell is.
- `www/templates/layout.typ`: scroll the floating wrapper's content instead of translating the clone's table, so the sticky cell has a scroll container to pin against. Restore its offset when a focus move scrolls the wrapper.


== Out of scope

- The floating header's border and shadow treatment beyond keeping the pinned cell readable. The live header keeps its shadow.
- The live header's first cell, already opaque.


== Dogfood

This repository's docs site under `www/` is the product's shop window. The comparison table is one of its most-read pages, and the floating header appears at any width narrower than the table. `just docs::check` builds every page.
