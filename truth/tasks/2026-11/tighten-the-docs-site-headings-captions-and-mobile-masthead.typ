#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Tighten the docs site headings, captions and mobile masthead",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("www", "docs", "stateless"),
  x-goals: (
    product: (
      readable-tasks: helps[the heading is the only label, and the prose starts at the title],
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
      docs-match-code: helps[`DESIGN.md` records the numbering and title treatment the site actually renders],
    ),
  ),
  status: done(2026, 11, 24),
  assignee: "agent",
)

== AS-IS

+ Serve the site with `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ Run `impeccable detect www/public/mindtape/index.html`.
+ 🔴 It reports `11px body text` for the section note and `11.52px body text` for the LSP source line.
+ Read the home page from the masthead to the rules table.
+ 🔴 Every section head carries a green `03` or `04` and a note such as `7 RECIPES · CONFIG, NOT CODE`, and every article h1 sits under a `01 RECIPES / …` chip.
+ 🔴 The terminal header and the LSP caption read `TERMINAL · examples/api-service` and `tinymist 0.14.10 · tasks/fix-the-login-redirect.typ`, with the path glued to the separator.
+ Resize the viewport to 390px.
+ 🔴 The masthead tools stop short of the right edge, and the article head keeps a 40px top pad above its title.

```text
$ impeccable detect www/public/mindtape/index.html 2>&1 | rg tiny-text
  [tiny-text] 11px body text
  [tiny-text] 11.52px body text
```


== TO-BE

- 🟢 `impeccable detect` passes the home page on `tiny-text`.
- 🟢 Section heads carry only their heading. The left line map keeps `01–07`, and no article h1 carries a chip above it.
- 🟢 Section notes disappear. Only the heading and the hairline remain above the prose.
- 🟢 Article heads start at the top of the shell, with no pad above the title.
- 🟢 At 390px the masthead tools sit on the right content edge.
- 🟢 The terminal header and the LSP caption put their label at the left and their path in a right-aligned field, with no middle dot.

```html
<div class="terminal-head"><i></i><span class="terminal-label">TERMINAL</span><span class="terminal-path">examples/api-service</span></div>
```


== Source of truth

- `www/utils/site.typ`: the section head, without a number or a note.
- `www/templates/layout.typ`: the article head, without a kicker.
- `www/utils/ui.typ`: the hero, the terminal head's label and path, and the LSP caption.
- `www/assets/styles/components.css` and `www/assets/styles/main.css`: the matching rules.
- `DESIGN.md`: the line-map numbering and the title treatment the site renders.


== Changes

- `www/utils/site.typ`: drop the `no` and `note` slots from `section-head`. The h2 is the whole head.
- `www/templates/layout.typ`: drop `_kicker` and `_page-no`. The article head starts at the h1.
- `www/utils/ui.typ`: drop the hero kicker. Split the terminal head and LSP caption into a label and a right-aligned path.
- `www/utils/demo.typ`: update the `head` callbacks to the new section-head shape.
- `www/content/index.typ`: update the six home section calls.
- `www/assets/styles/components.css`: remove the section-number and section-note rules. Right-align both paths and raise the LSP source line to 12px.
- `www/assets/styles/main.css`: remove the kicker rules, zero the article-head top pad, and pin the mobile tools to the right.
- `DESIGN.md`: state that numbering lives in the rail and no label sits above a title.


== Out of scope

- The three `low-contrast` detector advisories, which need their own palette decision.
- Routes, page copy and the generated `www/public/` build.
- The top route strip, which keeps its station labels.


== Dogfood

This repo's docs site is the surface: `just docs::serve` renders the accepted headings, captions and mobile masthead, and the detector reports none of the removed patterns.

