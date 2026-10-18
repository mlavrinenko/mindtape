#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Put the docs nav behind a menu button on narrow screens",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader on a phone sees the pitch and the page first, not a screen and a half of links],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 18),
)

== Problem

+ Run `just docs build`, serve `www/public/` and open the site root and `/config/` in a 375px wide viewport.
+ 🔴 Every sidebar group sits above the page, so the home hero starts about 900px down.
+ 🔴 `/config/`, `/cli/ls/` and twelve other pages scroll sideways: a table cell or a paragraph holds a code span or a URL wider than the screen.


== Environment

- The docs site built from `main` at the commit that filed this, in headless Chromium on Linux.


== TO-BE

- 🟢 Below the 52rem breakpoint the page comes first, and a menu button in the header, beside the GitHub mark and the theme switch, opens the nav as a drawer.
- 🟢 The button carries `aria-label` and `aria-expanded`. The drawer opens scrolled to the current page, closes on a link tap, a tap outside, Escape or the button, and locks the page scroll while open.
- 🟢 No page scrolls sideways at 375px. A wide `pre` or the comparison matrix still scrolls inside its own box.
- 🟢 Above the breakpoint the site renders as before.


== Changes

- `www/templates/layout.typ`: the menu button and a few lines of script beside the theme script.
- `www/assets/styles/main.css`: the button, the drawer and its lock, all under the narrow media query.
- `www/assets/styles/components.css`: wrap a long code span, URL or table cell on narrow screens.


== Out of scope

- The desktop sidebar and the search box. Both stay as they are.


== Dogfood

Not applicable: the docs site has no task data to dogfood, and the owner checks it on a phone.

