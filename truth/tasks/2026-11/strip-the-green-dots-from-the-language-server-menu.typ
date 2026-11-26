#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Strip the green dots from the language-server menu",
  kind: "fix",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("www", "stateless", "no-repro"),
  x-goals: (
    product: (
      readable-tasks: helps[the menu reads as field names, not prose],
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
      docs-match-code: helps[the stylesheet matches the component markup],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 22),
  links: edge(
    "surfaced-by",
    "rebuild-the-docs-site-as-a-transit-line.typ",
  )[The transit-line rebuild added the prose bullet rule over every list in main.],
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ Scroll to the section headed `Edit tasks with autocomplete.` and look at the completion menu over the empty line.
+ 🔴 Every menu row carries a 5px green square at its left edge. The field list reads as prose bullets. DevTools shows `.lsp-menu li::before` with a 5px by 5px box and `background-color: rgb(18, 128, 90)`.
+ 🔴 The diagnostics rows under the code get the same square from `main ul > li::before`.


== Environment

- The docs site built from `docs-redesign` at `aea991f`, opened in Chromium on Linux.
- `mt 0.6.0`.


== TO-BE

- 🟢 The completion menu shows `field` and the field name only, with no square at the row's left edge.
- 🟢 The diagnostics list below the code loses the same square.
- 🟢 An article's prose list keeps its green square, because a plain Typst list is content, not a component.

```
main ul:not([class]) > li::before {
  content: ""; position: absolute; left: -0.9rem; top: 0.72em;
  width: 5px; height: 5px; background: var(--green);
}
```


== Changes

- `www/assets/styles/main.css`: scope the green `::before` bullet to `main ul:not([class])`. A list that declares a component class draws its own markers, and the LSP menu and diagnostics list stop inheriting the prose bullet.


== Out of scope

- The prose bullet itself. Article lists keep the 5px green square.
- The line map, calling points and code headers, whose dots mark a real destination.


== Dogfood

This repo's docs home page carries the LSP demo, and `just docs::check` builds and validates the page.

