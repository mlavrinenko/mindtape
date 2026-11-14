#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Draw the dependency graph and the status moves as diagrams",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: helps[blocked work and a status legend read at a glance from a diagram],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[each build draws both from captured output, so they cannot drift],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 6),
)

== AS-IS

+ Open `/links/`.
+ 🔴 The page gives the edges between the fixture's tasks as text, and the reader assembles the graph in their head.
+ Open `/statuses/`.
+ 🔴 The page lists the moves each status allows as text. The shape of a legend, its loops and its dead ends, stays invisible.


== TO-BE

- 🟢 The Task links page shows the fixture's `mt graph` output drawn as an SVG.
- 🟢 The Statuses page shows the legend it documents as a diagram of the allowed moves, drawn from the config `mt config show --json` prints.
- 🟢 Every build regenerates both, and both render in the light and dark themes. A renderer missing from the dev shell fails the build.
- 🟢 Prefer a renderer that runs inside Typst. Otherwise `graphviz` joins `devShells.default`.


== Source of truth

- `mt graph --format dot` on the fixture the Task links page uses.
- `mt config show --json`: the statuses and their allowed moves.


== Changes

- `www/generate.sh` or the page's `www/generate-*.sh`: capture both.
- `www/utils/ui.typ`: a diagram helper.
- `www/content/links.typ`, `www/content/statuses.typ`: place the diagrams.
- `flake.nix`: `graphviz`, if Typst cannot draw DOT.


== Out of scope

- An SVG output from `mt graph`. `mt` prints data, and a renderer draws it.
- Diagrams on other pages.


== Dogfood

Not applicable: the diagrams document fixtures, not this repo's backlog.

