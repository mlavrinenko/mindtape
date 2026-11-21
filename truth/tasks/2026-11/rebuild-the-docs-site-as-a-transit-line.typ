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
  status: wip(2026, 11, 21),
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

- 🟢 Every page reads as a stop on one line: a route strip above, a line-map sidebar with the current stop marked, and a station-sign title.
- 🟢 The home page sets a real captured `mt ls` board beside the pitch. The mechanism shows before the first scroll.
- 🟢 Guide pages keep a 690px reading column and a calling-points rail, and the next stop closes each page.
- 🟢 Light and dark themes keep the cassette mark and its green accent, and search, the mobile drawer and the Markdown copy still work.

```
LINE 01  ● START  ○ RECIPES  ○ TASKS  ○ FIELDS  ○ CHECKS  ○ REFERENCE  ○ INTEGRATE
01 START / OVERVIEW
Mindtape
DEPARTURES / backlog                  9 OF 14 TASKS
MT LS / DEFAULT VIEW                        LIVE
```

== Changes

- `www/templates/layout.typ`: the masthead, route strip, line map, article head, calling-points rail, next stop and footer.
- `www/utils/site.typ`: the station label, the Markdown copy button and the package version tokens.
- `www/utils/ui.typ`: the hero, the departure board, the source panel and the section head.
- `www/utils/demo.typ`: the site and Markdown variants of the recipe table and the Typst section.
- `www/content/index.typ`: the six numbered home sections and their copy.
- `www/assets/styles/main.css`: tokens, faces, chrome, line map, article layout, prose, motion and responsive rules.
- `www/assets/styles/components.css`: hero, board, code panels, calling points, callouts, tables and search.
- `www/assets/fonts/archivo-latin.woff2`: the self-hosted display face, with text and mono subsets beside it and their open font licenses.
- `www/assets/images/github.svg`: the masthead mark.
- `www/generate-reference.sh`: the `mt ls` board capture and its JSON extraction.
- `www/post-build.sh`: strips tola's hidden enhance stylesheet from the built pages.
- `www/capture-mirror.sh`: mirrors the built assets for the static capture gate.
- `.just/docs.just`: runs the post-build step after tola.
- `.gitignore`: ignores the generated capture mirror.
- `PRODUCT.md`: the durable product and brand context for the redesign.
- `DESIGN.md`: the extracted Line design system and its tokens.

== Out of scope

- The Impeccable records under `.impeccable/`. The user decides what to keep in a later session.
- The capture mirror and the hero manifest. They serve the capture gate only, and the site does not read them.
- An alternate display face. The approved comp renders in the site's sans, and the build matches its pixels.

== Dogfood

This repository's docs site is the product's shop window. The new shell runs on every page under `www/`, the home board reads captured `mt ls` output from `www/generated/`, and `just docs::serve` serves the result. The Pages workflow deploys the same build.
