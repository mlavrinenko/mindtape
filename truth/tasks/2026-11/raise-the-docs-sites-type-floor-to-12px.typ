#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Raise the docs site's type floor to 12px",
  kind: "fix",
  priority: ice(
    impact: 6,
    confidence: 0.95,
    ease: 8,
  ),
  difficulty: 2,
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
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[`DESIGN.md` records the 12px reading floor and its uppercase-label exception],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 25),
)

== AS-IS

+ Run `just docs::build`, serve `www/public`, and open the site at 1280px and 390px.
+ 🔴 A computed-style scan of all 61 pages finds 6,102 text nodes below 12px. Article code panels render at 11.5px. Terminals and table cells render at 11px. Inline code in list items renders at 11.2px, and matrix row labels at 11.84px.
+ 🔴 The code-panel path, terminal path, LSP caption, sidebar sub-stops, rail foot and footer links also render at 11px.
+ 🔴 Only the uppercase labels and the `/` search hint belong at 11px, as the design's label role.

```
> getComputedStyle(document.querySelector(".prose pre code")).fontSize
"11.5px"
> getComputedStyle(document.querySelector("main table code")).fontSize
"11px"
> getComputedStyle(document.querySelector(".matrix tbody td:first-child")).fontSize
"11.84px"
```


== Environment

- The docs site built from `docs-redesign` at 60b3d3c5, rendered in Chromium 157 on Linux.
- `mt 0.6.0`.


== TO-BE

- 🟢 Every reading and data role renders at 12px or larger. That includes article code panels, terminals, inline code in prose and lists, table code and matrix labels. It also includes code and terminal headers, the LSP caption and the footer.
- 🟢 The only text below 12px is the uppercase label role (stations, nav titles, table heads, buttons, `COPY`, `TERMINAL`, `Copy as Markdown`) and the one-character `/` search hint.
- 🟢 `DESIGN.md` records the 12px reading floor and that uppercase-label exception.
- 🟢 The code-panel header still keeps `COPY` on screen at 390px.

```
.prose pre, .prose .code pre { font-size: 12px; }
.prose code { font-size: max(12px, 0.8em); }
main table code { font-size: 12px; }
```


== Changes

- `www/assets/styles/main.css`: raise inline code, fenced code, terminal code, sub-stops, rail foot and footer text to 12px.
- `www/assets/styles/components.css`: raise the code and terminal headers, table code, matrix labels, LSP caption, kind and severity, search clear and search message to 12px.
- `DESIGN.md`: record the 12px reading floor and the uppercase-label exception.


== Out of scope

- Raising the uppercase 11px label role. It stays at 11px by design and is the allowed exception.
- Adding a gate that enforces the floor. This task verifies the built site by computed style.
- Reworking type sizes already at or above 12px.


== Dogfood

This repository's docs site under `www/` is the product's shop window. `just docs::check` builds every page, and the computed-style audit covers all 61 pages.

