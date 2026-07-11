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
  status: proposed(2026, 7, 11),
)

== Summary

Follow-up to generate-docs-cli-reference, which ships plain-text injected
transcripts. Add colour without touching any content page: the generator
captures ANSI and converts it to HTML, and `terminal()` renders the spans. The
two-seam change is only possible because generate-docs routes every capture
through the generator and every render through `terminal(read(...))`.

== Approach

- Generator: capture with `--color always`, convert ANSI → HTML spans (an
  `ansi-to-html` crate or `aha`), write `www/generated/<slug>.html` (or annotate
  the `.txt`).
- `terminal()` (`www/utils/ui.typ`): gain an ANSI/HTML-aware path that renders
  the spans as raw HTML in the terminal panel; map ANSI colours to CSS variables
  so light/dark themes both work.
- No content-page edits — they already call `terminal(read(...))`.

== Scope

- generator colour capture + ANSI → HTML conversion.
- `terminal()` span rendering + theme-aware CSS colour mapping.
- verify a sampling (`mt ls` bands, `mt check` ok/fail) renders coloured in both
  light and dark.
