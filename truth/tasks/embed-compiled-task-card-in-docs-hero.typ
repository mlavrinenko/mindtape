#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Embed a compiled task card in the docs hero",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 7,
  ),
  tags: ("docs",),
  links: depends-on("render-task-files-as-typeset-documents.typ")
    + depends-on("generate-docs-cli-reference-from-the-mt-binary.typ"),
  status: done(
    2026,
    7,
    12,
  )[Hero card face now compiles fur-elise-polish.typ to SVG (throwaway `--package-path`) and inlines it as a data-uri `<img>` via `image(read(…, encoding: none))` — dodging tola's asset validator since generated/ is never copied. Hand-built task-card/pill/tag helpers and CSS dropped.],
)

== Summary

Close the loop the render task opened. The landing hero shows three faces —
source, terminal, typeset — but the third ("typeset · rendered") is still a
hand-built HTML `task-card` (`www/utils/ui.typ`) faking a `wire-oauth` task. Now
that the `task` show rule renders a real card (render-task-files-as-typeset), and
that the docs generator already routes every face through a build-time capture,
the third face should be a genuinely compiled fixture task — the same
`fur-elise-polish.typ` the source face reads — not a hand-authored fake.

== Approach

- Generator (`www/generate.sh`): `typst compile` the hero fixture task to SVG
  under `www/generated/` (self-contained `--package-path` symlink to `lib/`, no
  reliance on `just install-lib`). Build-time only, gitignored like the rest.
- Hero (`www/utils/ui.typ`): the card face reads the compiled SVG via
  `image("../generated/hero-card.svg")` — Typst HTML export inlines it as a
  self-contained `data:image/svg+xml;base64` `<img>`, so no asset plumbing and
  nothing lands in `public/`. Drop the hand-built `task-card` / `pill` / `tag`
  helpers and their `wire-oauth` literals.
- CSS: the `.face-card .face-body` reset only targets `> pre`/`> .terminal`/`>
  .task-card`; add a rule for the new `<img>` child so it fits the column.

== Scope

- generator SVG capture + hero `image()` embed; remove the dead hand-built card.
- verify `just docs-build` inlines the SVG and the hero renders the real task.
