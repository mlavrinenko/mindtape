#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Add the next stop to the docs home page",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 10,
  ),
  difficulty: 1,
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
      docs-match-code: helps[the front page ends at the same next stop as every article],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 25),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ Scroll to the end of the front page, past the "For people and for agents." section.
+ 🔴 The page ends at that section's last paragraph, with no next-stop block, while every article ends with one.

== TO-BE

- 🟢 The front page ends with the same next-stop block as an article, pointing at Quickstart.
- 🟢 The route label reads `01 START`, the label `Quickstart`, and the arrow points right.

```html
<a class="next-stop" href="/mindtape/quickstart"><span class="next-route">01 START</span><span class="next-label">Quickstart</span><span class="next-arrow">→</span></a>
```

== Source of truth

- `www/templates/layout.typ`: `_next-stop`, the only writer of the block, and `_reading-order`, which starts at `/`.
- `www/content/index.typ`: the `home: true` meta that selects the branch with no block.

== Changes

- `www/templates/layout.typ`: render `_next-stop(active)` after the home page's body.

== Out of scope

- The block's width on the home page. It takes the home content column, which is wider than an article's prose column.
- The reading order and the last page's missing block. `truth/tasks/2026-11/drop-the-next-stop-from-the-docs-last-page.typ` settled both.

== Dogfood

Every page but the last ends at a next stop, and `just docs::check` builds and validates the front page.
