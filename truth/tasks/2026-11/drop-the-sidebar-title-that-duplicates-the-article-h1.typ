#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Drop the sidebar title that duplicates the article h1",
  kind: "fix",
  priority: ice(
    impact: 3,
    confidence: 0.98,
    ease: 10,
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
      self-describing: helps[a sidebar that repeats the h1 slows a newcomer scanning the docs for how to drive the CLI],
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

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/concepts/`.
+ 🔴 The rail head reads "Documentation" and "Concepts". The article h1 also reads "Concepts". The page title appears twice.

```
$ rg -o 'rail-head.{0,80}' www/public/mindtape/concepts/index.html
<div class="rail-head"><span class="eyebrow">Documentation</span><strong>Concepts</strong></div>
```


== Environment

- The docs site built from `docs-redesign` at 1e4d4243, in Chromium on Linux.
- `mt 0.6.0`.


== TO-BE

- 🟢 The rail head shows only the "Documentation" eyebrow. The page title appears once, as the article h1.
- 🟢 Every page's nav starts directly under the eyebrow, with no label between them.

```html
<div class="rail-head"><span class="eyebrow">Documentation</span></div>
```


== Changes

- `www/templates/layout.typ`: drop the current-page label from `_sidebar` and its unused `current` lookup.
- `www/assets/styles/main.css`: remove the `.rail-head strong` rule, which no element matches.


== Out of scope

- The "Documentation" eyebrow and every other sidebar label.
- The article h1, the page rail and the mobile drawer.


== Dogfood

This repository's docs site renders the rail on every page, so `just docs::check` serves a site where each title appears once.

