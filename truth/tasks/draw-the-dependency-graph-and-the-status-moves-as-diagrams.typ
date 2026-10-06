#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

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
      readable-tasks: helps[the shape of blocked work and of a status legend reads at a glance instead of from a list],
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[both diagrams are drawn from captured output each build, so they cannot drift],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 6),
)

== Problem

+ Open `/links/`.
+ 🔴 The edges between the fixture's tasks are given as text, and the reader assembles the graph in their head.
+ Open `/statuses/`.
+ 🔴 The moves each status allows are listed as text, so the shape of a legend, its loops and its dead ends, is invisible.


== TO-BE

- 🟢 The Task links page shows the fixture's `mt graph` output drawn as an SVG.
- 🟢 The Statuses page shows the legend it documents as a diagram of the allowed moves, drawn from the config `mt config show --json` prints.
- 🟢 Both are regenerated in every build and read in the light and dark themes. A renderer missing from the dev shell fails the build.
- 🟢 A renderer that runs inside Typst is preferred. Otherwise graphviz joins `devShells.default`.


== Source of truth

- `mt graph --format dot` on the fixture the Task links page uses.
- `mt config show --json`: the statuses and their allowed moves.


== Changes

- `www/generate.sh` or the page's `www/generate-*.sh`: capture both.
- `www/utils/ui.typ`: a diagram helper.
- `www/content/links.typ`, `www/content/statuses.typ`: place the diagrams.
- `flake.nix`: graphviz, if Typst cannot draw DOT.


== Out of scope

- An SVG output from `mt graph`. `mt` prints data, and a renderer draws it.
- Diagrams on other pages.


== Dogfood

Not applicable: the diagrams document fixtures, not this repo's backlog.

