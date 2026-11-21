#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Raise the docs site's type floor to 11px",
  kind: "fix",
  priority: ice(
    impact: 7,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
  estimate: (src-files: 3),
  tags: ("www", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[every label is legible at first glance],
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
      docs-match-code: helps[`DESIGN.md` records the 11px floor the CSS uses],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
  links: edge(
    "surfaced-by",
    "rebuild-the-docs-site-as-a-transit-line.typ",
  )[The rebuilt site rendered labels at 8-9px.],
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`, then inspect the masthead, sidebar, home board and an article.

+ 🔴 Chromium reports 7.5px for the Markdown copy button, 8px for sidebar group numbers, and 8px for board metadata.

+ 🔴 It reports 8.5px for board statuses and the LSP caption, and 9px for the board and code headers.

+ 🔴 Nav titles, the search key and section numbers render at 10px. CLI sub-stops render at 10.5px.

+ 🔴 Inline code inside an article code panel renders at 9.2px. The `.prose code` rule's `0.8em` beats `pre code`'s `inherit`. The code shrinks below the panel's 11.5px.

```
$ just docs::serve
# DevTools console on the served page:
> getComputedStyle(document.querySelector(".group-no")).fontSize
"8px"
> getComputedStyle(document.querySelector(".prose pre code")).fontSize
"9.2px"
```


== Environment

- The docs site built from `docs-redesign` at dc37010, rendered in Chromium 155 on Linux.
- `mt 0.6.0`.


== TO-BE

- 🟢 Every text role on the site renders at 11px or larger. Labels stay distinct by family, weight, tracking and color rather than shrinking.
- 🟢 Inline code inside an article code panel keeps the panel's size: 11.5px, or 11px in a terminal.
- 🟢 Inline code inside a table or callout never drops below 11px.
- 🟢 `DESIGN.md` records the floor, so a later typography pass does not reintroduce 8–9px labels.
- 🟢 The hero code header keeps its `COPY` control on screen at 390px.

```
.section-no, .st, .code-head, main thead th { font-size: 11px; }
.prose code { font-size: max(11px, 0.8em); }
.prose pre code { font-size: inherit; }
```


== Changes

- `www/assets/styles/main.css`: raise each sub-11 label role to 11px. Floor inline code with `max(11px, 0.8em)`, and let code inside `pre` inherit the panel size.
- `www/assets/styles/components.css`: raise the board, code header, terminal, table, LSP and search labels to 11px. Unwrap the hero code header's Typst `<p>` so the path truncates and `COPY` stays visible.
- `DESIGN.md`: set the label token to 11px, update the hierarchy and board notes, and add the Legibility Floor rule.


== Out of scope

- A gate in `docs::check` that enforces the floor. The user declined one.
- Reworking the remaining 12–15px roles that sit off the compact `DESIGN.md` ramp.
- The ignored `.impeccable/` records, which are the user's to decide.


== Dogfood

This repository's docs site under `www/` is the product's shop window. Every page renders at the floor. `just docs::check` builds it.

