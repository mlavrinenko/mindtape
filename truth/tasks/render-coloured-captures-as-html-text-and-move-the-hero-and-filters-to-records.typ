#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Render coloured captures as HTML text and move the hero and /filters/ to records",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("docs", "www"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the hero and /filters/ print the command that produced their output],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 3),
)

== Problem

+ Open the home page and `/filters/`.
+ 🔴 The hero's `mt ls` and the `/filters/` listing are SVG images drawn from `.ansi` files that `www/generate.sh` captures off a terminal with `--color always`. They show no status glyphs, and the page does not show the command.
+ Run `typst compile --ignore-system-fonts` on a text block holding `💡`.
+ 🔴 The emoji is missing from the SVG. A record from `www/capture.sh` runs in a terminal and carries status glyphs, so `terminal-ansi()` would show them only on a build machine with an emoji font.


== TO-BE

- 🟢 `terminal-ansi(<slug>)` reads a record and renders each SGR run as an HTML span with an inline colour, the way Typst's HTML export renders syntax highlighting. The text stays selectable, and the browser draws the glyphs.
- 🟢 The hero and `/filters/` read records, so both print the command they show.
- 🟢 `www/generate.sh` writes no `.ansi` file.


== Source of truth

- The records `www/capture.sh` writes, and `terminal()` in `www/utils/ui.typ`.
- The HTML Typst emits for a `raw(..., lang: "sh")` block: `<span style="color: …">` per token.


== Changes

- `www/utils/ui.typ`: `terminal-ansi(slug)` renders HTML spans; drop `_ansi-block`, the `html.frame` and the `.sr-only` twin.
- `www/generate.sh`: capture the hero's `mt ls` and the `!terminal` listing as records; drop `gen_color`.
- `www/content/filters.typ` and the hero in `www/utils/ui.typ`: read the records.
- `www/assets/styles/components.css`: drop the `.terminal-svg` rules.


== Out of scope

- Plain transcripts: `terminal()` stays as it is.


== Dogfood

This repo's home page shows a real `mt ls` and the command that printed it on every `just check`.

