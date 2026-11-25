#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Let the docs menu drawer scroll on narrow screens",
  kind: "fix",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("www", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: helps[a tablet reader reaches the install and quickstart pages from the menu.],
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
  status: done(
    2026,
    11,
    25,
  )[The fixed drawer fills the space under the masthead. Its overflow-y scrolls at 768 and 390. Browser checks, docs check and the build gate pass.],
)

== AS-IS

+ Run `just docs build`, serve `www/public/` and open `/mindtape/dsl/` at 768 by 1024.
+ Tap the menu button to open the drawer. Its last group, Internals, sits past the bottom edge.
+ 🔴 Drag or wheel inside the drawer. It does not move, and its content never overflows its own box:

```js
// In the page's console, with the drawer open:
const s = document.querySelector('.sidebar');
[s.clientHeight, s.scrollHeight]  // [1265, 1265]
```


== Environment

- The docs site built from `docs-redesign` at the commit that filed this, in headless Chromium 768 by 1024, on Linux. The site has no browser test harness, so the check ran in the browser.


== TO-BE

- 🟢 The open drawer spans from the masthead to the bottom of the viewport, and its list scrolls by touch, wheel and keyboard until the last group is reachable.
- 🟢 The desktop sidebar and the drawer's focus trap stay as they are.

```js
const s = document.querySelector('.sidebar');
[s.clientHeight, s.scrollHeight]  // [960, 1265]
s.scrollTop = 9999;               // clamps to 305
```


== Changes

- `www/assets/styles/main.css`: reset `align-self` on the fixed `.js-nav .sidebar` under the 52rem media query. A comment there explains why the base rule's `align-self: start` leaves the drawer content-sized, and the reset lets `top` and `bottom` fix the drawer's height so `overflow-y: auto` scrolls.


== Out of scope

- The desktop sticky sidebar and its own scrolling, which work as they are.
- The drawer's focus trap, the search drawer and the scrim, which this fix leaves untouched.


== Dogfood

This repo's own docs menu: a reader on a tablet can reach every group, including CLI and Internals, without resizing the window.

