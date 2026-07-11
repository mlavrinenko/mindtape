#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Structured CLI reference from a command dump",
  priority: ice(
    impact: 4,
    confidence: 0.6,
    ease: 4,
  ),
  tags: ("docs",),
  links: depends-on("generate-docs-cli-reference-from-the-mt-binary.typ"),
  status: proposed(2026, 7, 11),
)

== Summary

Follow-up to generate-docs-cli-reference, which injects verbatim `mt --help`
blocks. Replace them with a styled, structured reference generated from a
machine-readable command tree, so the CLI page renders grouped commands and
flags in the site's visual language while staying binary-sourced. This renders
the static help tree only — not the `mt ls` data table (that re-implementation
was ruled out).

== Approach

- Emit the command tree as JSON: a hidden `mt _dump-cli --json` walking
  `Cli::command()` (names, abouts, args, value names, defaults), or an equivalent
  clap walker in the generator. Write `www/generated/cli.json`.
- `www/content/cli.typ` renders the reference from `cli.json` (grouping +
  styling), replacing the verbatim `--help` blocks.
- `mt --version` and the sample transcripts stay verbatim from generate-docs.

== Constraints

- Binary-sourced: no hand-typed flag facts; the JSON schema is the contract.

== Scope

- `mt _dump-cli --json` (or a generator-side clap walker) → `cli.json`.
- a Typst renderer for the command tree on `cli.typ`.
- keep `--version` and sample transcripts as they are (still verbatim).
