#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Drop the Next stop caption from the docs page footer",
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
      docs-match-code: helps[the stylesheet stops styling a caption the markup stopped rendering],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 24),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/cli/`.
+ Scroll past the article to the block above the footer.
+ 🔴 The block opens with a green `Next stop` caption. The label repeats what the arrow already says.


== TO-BE

- 🟢 The block opens with the target page's route and title, such as `06 REFERENCE` and `mt add`.
- 🟢 The green dot, the link's full-width hit area and the arrow keep their places.

```html
<a class="next-stop" href="/mindtape/cli/add"><span class="next-route">06 REFERENCE</span><span class="next-label">mt add</span><span class="next-arrow">→</span></a>
```


== Source of truth

- `www/templates/layout.typ`: `_next-stop`, the only writer of the block.
- `www/assets/styles/main.css`: the `.next-stop` rules and the `.next-kicker` rule.


== Changes

- `www/templates/layout.typ`: drop the `next-kicker` span, leaving the route, title and arrow.
- `www/assets/styles/main.css`: drop the rule that styled it.


== Out of scope

- The reading order and the missing block on the last page. `truth/tasks/2026-11/drop-the-next-stop-from-the-docs-last-page.typ` settled both.
- The block's dot, arrow and hover colors.


== Dogfood

Every page but the last renders this block, and `just docs::check` builds and validates them all.

