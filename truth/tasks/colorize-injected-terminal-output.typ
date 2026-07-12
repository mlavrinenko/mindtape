#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Colorize injected terminal output",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 6,
  ),
  tags: ("docs",),
  links: depends-on("generate-docs-cli-reference-from-the-mt-binary.typ"),
  status: wip(2026, 7, 12),
)

== Summary

Follow-up to generate-docs-cli-reference, which ships plain-text injected
transcripts. Add colour without touching any content page: the generator
captures ANSI and converts it to HTML, and `terminal()` renders the spans. The
two-seam change is only possible because generate-docs routes every capture
through the generator and every render through `terminal(read(...))`.

== Constraint (discovered)

The original two-seam plan (ANSI → HTML `<span>`, injected raw) is not
possible: tola / Typst HTML export exposes no raw-HTML-string escape hatch
(`html.elem` escapes its text), and `text(fill: …)` colours are dropped in HTML
export. The one colour-carrying path is `html.frame(content)`, which renders
native Typst content to inline `<svg>`. The terminal panel background is
theme-invariant (`--term-bg` has no dark override), so baked SVG colours read
correctly under both site themes — no CSS-variable colour mapping needed. Cost:
the transcript renders as an image, so the OUTPUT is not text-selectable (the
copyable commands stay real text in the sibling `sh` code blocks).

== Approach

- Generator (`www/generate.sh`): capture the colour-bearing transcripts a second
  time with `--color always` → `www/generated/<slug>.ansi` (raw SGR bytes).
- `terminal()` (`www/utils/ui.typ`): an ANSI-aware path parses the SGR runs in
  Typst, builds a monospace block of `text(fill: …)` coloured runs (One-Dark
  palette, readable on the dark panel; SGR reset → `--term-fg`), and wraps it in
  `html.frame(…)` so it serializes to inline SVG inside the `.terminal` div.
- No content-page rewrites of the transcript source — pages keep calling the
  `terminal` helper; only the helper and its capture change.

== Scope

- generator `--color always` capture for the colour-bearing transcripts.
- `terminal()` ANSI parse → `text(fill:)` runs → `html.frame` SVG render.
- verify `mt ls` bands and `mt check` render coloured, and the panel reads in
  both light and dark.
