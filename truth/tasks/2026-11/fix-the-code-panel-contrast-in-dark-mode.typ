#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Fix the code panel contrast in dark mode",
  kind: "fix",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 1,
  tags: ("www", "docs", "stateless", "no-repro"),
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
      docs-match-code: helps[the code panel in the stylesheet and the design rule change together],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 11, 21),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/` in dark mode.
+ Open a page with a code panel, such as the DSL page.
+ 🔴 The panel keeps light paper (`#f3f5f0`) on a night ground (`#0f1712`), so it reads as a white island. Typst's paper palette stays on it: red `#d73948`, blue `#4b69c6`, grey `#6b6b6f`, pink `#b60157`, purple `#8b41b1` and teal `#1d6c76`. All six sit between 2.34:1 and 3.41:1 against the panel, under the 4.5:1 floor.
+ 🔴 On the light paper, red `#d73948`, green `#198810` and grey `#74747c` sit between 4.19:1 and 4.22:1, also under the 4.5:1 floor.

== Environment

- The docs site at `197011e2` on `docs-redesign`, opened in Chromium on Linux.

== TO-BE

- 🟢 In dark mode, a code panel sits on a raised night panel (`#1a261e`) with a `#2c3a32` border. Eight dark overrides remap the baked palette, and the new values run from 5.15:1 to 9.43:1 against that panel.
- 🟢 In light mode, red `#c93240` (4.77:1), green `#177c0e` (4.87:1) and grey `#6d6d72` (4.69:1) clear the 4.5:1 floor.
- 🟢 The selectors scope to `pre span`, so terminal ANSI output keeps its own palette. `DESIGN.md` states the night panel rule instead of the light paper rule.

```
:root {
  --code-bg: #f3f5f0;
}

[data-theme="dark"] {
  --code-bg: #1a261e;
  --code-border: #2c3a32;
}
```

== Changes

- `www/assets/styles/main.css`: dark-mode `--code-bg`, `--code-fg` and `--code-border` tokens, eight `[data-theme="dark"] pre span` color overrides, and three light-mode `pre span` overrides to clear the 4.5:1 floor.
- `DESIGN.md`: the Overview sentence, the Code Panel style and the Do bullet say code sits on a night panel in dark mode.
- `truth/tasks/2026-11/rebuild-the-docs-site-as-a-transit-line.typ`: drop the out-of-scope line that ruled out a dark code paper.

== Out of scope

- The other dark-mode surfaces and tokens. The page ground, chrome and terminal panel already pass and keep their colors.
- Typst's syntax theme. The site remaps the baked span colors in CSS, and the generated HTML keeps Typst's palette.

== Dogfood

This repository's docs site builds from `www/`, so every page with a code panel shows the night panel in dark mode. `just docs build` and the Pages workflow deploy it.
