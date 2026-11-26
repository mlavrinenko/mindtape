#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Track the current calling point while the reader scrolls",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
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
      docs-match-code: helps[`DESIGN.md` and the site CSS specify the same current-stop behavior],
    ),
  ),
  assignee: "agent",
  status: done(2026, 11, 22),
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/concepts/`.
+ 🔴 The calling-points rail marks the first heading green and leaves every later stop hollow.
+ 🔴 Scrolling to `Identity` or clicking it leaves the first stop green and every later stop hollow. The rail never says where the reader is.


== TO-BE

- 🟢 The stop for the heading under the masthead is current: green text, a filled dot with a paper ring, and `aria-current="location"`.
- 🟢 Every stop above it shows a filled green dot. Every stop below stays hollow.
- 🟢 Each dot centres on its label, and the green segment ends at that centre.
- 🟢 A green segment runs from the top of the list to the current dot, and the rest of the line stays grey.
- 🟢 The rail follows the reader on scroll, and a click moves the mark at once and holds it through the smooth scroll until the reader scrolls again.
- 🟢 A page opened at a `#fragment` marks that section, and the narrow folded list marks the same stop.

```
CALLING POINTS
●  A task is a file
●  Every command reads the files
◉  Identity
○  Links name files
○  Extension fields
```


== Source of truth

- `www/templates/layout.typ`: `_toc-script` builds the rail from the rendered `h2[id]` headings.
- `www/assets/styles/components.css`: the `.page-toc` dot and line rules.
- `www/content/concepts.typ`: the seven-section page that exercises the rail.


== Changes

- `www/templates/layout.typ`: track the heading under the masthead on scroll, set `is-passed` and `is-current`, write `--toc-here` at the current label's centre, and hold a clicked stop through its smooth scroll.
- `www/assets/styles/components.css`: center every stop on its label, replace the first-child highlight with current and passed rules, and draw the green segment over the grey line.
- `DESIGN.md`: say the rail fills passed and current stops and draws a green segment to the current stop.


== Out of scope

- The line map's route-derived highlight. It marks a page, not a heading.
- The heading markup and slug ids. The script reads them as they are.
- A browser test runner. The site has none. The gate stays `just docs::check`, and the behavior gets a manual check.


== Dogfood

This repository's docs site is the first reader: `just docs::check` builds every page, and `just docs::serve` plus `/concepts/` shows the rail following the reader.

