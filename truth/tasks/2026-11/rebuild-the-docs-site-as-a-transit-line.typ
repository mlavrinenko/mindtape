#import "/truth/prelude/1/lib.typ": *

#show: task.with(
  title: "Rebuild the docs site as a transit line",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 4,
  ),
  difficulty: 5,
  kind: "milestone",
  tags: ("www", "stateless"),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      config-over-code: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: helps[the site uses Typst and shows the DSL],
      plain-files: neutral,
      self-describing: helps[the docs show the mechanism on the first screen],
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
    26,
  )[The final shell uses the cassette hero mark and no line number. Follow-ups added night code panels and a 12px floor.],
)

== AS-IS

+ Run `just docs::serve` and open `http://127.0.0.1:5277/mindtape/`.
+ 🔴 The home page reads as a generic docs shell: one top bar, a flat link list and prose. Nothing on the first screen shows a real task, and every guide page wears the same frame with no sense of place.

```
<header>Mindtape · Docs</header>
<aside>Start / Recipes / Tasks / Fields / Checks</aside>
<main>Install Mindtape...</main>
```

== TO-BE

- 🟢 Every page reads as a stop on one line: a route strip, a numbered line-map sidebar marking the current stop, and a station-sign title.
- 🟢 The home hero sets the name across the full width with the cassette mark at its edge. Below it, a real task file, its `mt ls` listing and its typeset card show the mechanism.
- 🟢 Article pages keep a 690px reading column.
- 🟢 A page with three or more headings gets a scroll-tracked calling-points rail, and every page but the last ends at the next stop.
- 🟢 Light and dark themes keep the cassette mark and green accent, code panels turn night-dark in dark mode, and search, the mobile drawer and the Markdown copy still work.

```
● START  ○ RECIPES  ○ TASKS  ○ FIELDS  ○ CHECKS  ○ REFERENCE  ○ INTEGRATE
Mindtape                                       [cassette mark]
HERE'S A REAL TASK.   TASK FILE  examples/api-service/tasks/throttle-failed-logins-per-ip.typ
AND HERE'S HOW MT LS LISTS IT.   TERMINAL
NEXT STOP  01 START / QUICKSTART  →
```

== Changes

- `www/templates/layout.typ`: the masthead, route strip, numbered line map, article head, calling-points rail, next stop and footer.
- `www/utils/site.typ`: the nav model, station labels, section heads, Markdown copy button and release version.
- `www/utils/ui.typ`: the hero, its proof task, the terminal listing, callouts and the language-server demo.
- `www/utils/demo.typ`: the site and Markdown variants of the recipe table and the Typst section.
- `www/content/index.typ`: the home hero and the sections below it.
- `www/assets/styles/main.css`: tokens, faces, chrome, line map, article layout, prose, motion and responsive rules.
- `www/assets/styles/components.css`: hero, proof panels, calling points, callouts, tables and search.
- `www/assets/fonts/`: three self-hosted Latin subsets and their open font licenses.
- `www/assets/images/github.svg`: the masthead mark.
- `www/generate.sh`: calls the page-margin crop for the inline typeset card.
- `www/generate-reference.sh`: the proof terminal's `mt ls` capture.
- `www/crop-page-margin.py`: the typeset card's cropped page margin.
- `www/post-build.sh`: strips tola's enhance stylesheet, then fails the build on a missing release version or a page-count rail.
- `www/capture-mirror.sh`: mirrors the built assets for the static capture gate.
- `.just/docs.just`: runs the post-build step after tola.
- `.gitignore`: ignores the capture mirror and the Impeccable records.
- `docs/src/contributing/docs-site-build.typ`: records the post-build step.
- `PRODUCT.md`: the durable product and brand context for the redesign.
- `DESIGN.md`: the extracted Line design system and its tokens.

== Out of scope

- The Impeccable records under `.impeccable/`. The user decides what to keep in a later session.
- The capture mirror under `/mindtape/`. It serves the static capture gate only, and the site does not link it.
- An alternate display face. The self-hosted display face is the only one, and the build matches the approved comp.

== Dogfood

This repository's docs site is the product's shop window. The new shell runs on every page under `www/`, the home proof reads captured task and `mt ls` output from `www/generated/`, and `just docs::serve` serves the result. The Pages workflow deploys the same build.
