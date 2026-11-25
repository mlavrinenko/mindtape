#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Tighten the docs site's panels, rail and article lede",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.95,
    ease: 9,
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
      self-describing: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[`DESIGN.md` records the 40px header and the full width of the article lede],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 25),
)

== AS-IS

+ Check out `docs-redesign` at 60b3d3c5, run `just docs::build`, and open `/concepts/` at 1280px and 390px.
+ 🔴 The `.code` and `.terminal` rules omit the vertical margin. A panel sits flush against the prose around it.
+ 🔴 `.code-head` is 36px tall and `.terminal-head` is 34px tall around 12px labels.
+ 🔴 `.article-lede` centers against the Copy as Markdown control, and `.article-head .lede` caps the summary at `60ch`.
+ 🔴 The calling-points label draws a second top rule and 12px pad under the article head's rule. `.lsp-problems` pads 0.4rem above its first row, and `.nav-title` starts at 1px.

```
> getComputedStyle(document.querySelector(".code-head")).height
"36px"
> getComputedStyle(document.querySelector(".terminal-head")).height
"34px"
> getComputedStyle(document.querySelector(".article-head .lede")).maxWidth
"60ch"
```


== Environment

- The docs site built from `docs-redesign` at 60b3d3c5, rendered in Chromium 157 on Linux.
- `mt 0.6.0`.


== TO-BE

- 🟢 Code panels and terminals sit on a 20px vertical margin, separated from the prose above and below.
- 🟢 Both headers are 40px tall, matching the 12px labels and the copy control.
- 🟢 The article lede top-aligns with Copy as Markdown and fills the article head instead of capping at 60ch.
- 🟢 The calling-points label carries no duplicate rule or top pad.
- 🟢 The LSP problem list pads 0.3rem above its first row, and the nav title aligns with its group content.

```
.code, .terminal { margin: 20px 0; }
.code-head, .terminal-head { height: 40px; }
.article-lede { align-items: start; }
.article-head .lede { /* no max-width */ }
```


== Changes

- `www/assets/styles/components.css`: add the panel and terminal margins, set both headers to 40px, drop the calling-points rule, and tighten the LSP problem list.
- `www/assets/styles/main.css`: top-align the article lede, remove its 60ch cap, and pad the nav title by 2px.
- `DESIGN.md`: record the 40px code header, the panel margin, the full-width lede and the calling-points rail without its own rule.


== Out of scope

- The 12px type floor. `raise-the-docs-sites-type-floor-to-12px` owns it.
- The panel colors, borders and the copy control's behavior.
- Adding a gate for layout values. This task verifies the built site in a browser.


== Dogfood

This repository's docs site under `www/` is the product's shop window. `just docs::check` builds every page, and `/concepts/` shows the panels, calling points and article lede at both widths.

