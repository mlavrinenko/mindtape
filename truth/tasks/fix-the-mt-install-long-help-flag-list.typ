#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "fix the mt install long help flag list",
  priority: ice(
    impact: 7,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("cli", "docs"),
  estimate: (files: 1),
  links: parent("de-slop-the-docs-and-close-the-drift-seams.typ"),
  status: accepted(2026, 8, 16),
)

== Summary

`mt install`'s clap long help says it "performs both by default; `--no-lib`/
`--no-defaults` skip either half". It performs three installs — the prelude,
the global defaults, and the JSON Schema — and `crates/mindtape-cli/src/install.rs`
declares a third skip flag, `--no-schema`, that the help never mentions. Its own
module doc already says "By default performs all three installs".

The docs site renders that stale sentence out of `www/generated/cli.json`
directly above `www/utils/cli-notes.typ`'s correct text, which names all three
flags. So the CLI reference contradicts itself on one page, under a heading
that promises nothing there can drift from what `mt` actually is.

== Scope

`crates/mindtape-cli/src/cli.rs` — the `Install` variant's doc comment. The
site heals itself on the next `just docs gen`; nothing under `www/` needs an
edit.

== Note

This is the failure mode the derived CLI reference was supposed to make
impossible, and it is worth seeing clearly: deriving the reference from clap
moved the drift out of the docs and into the doc comment. A doc comment is
prose with no gate on it. Worth asking, separately, whether a flag added to a
command's arg struct without appearing in that command's long help is
checkable.
