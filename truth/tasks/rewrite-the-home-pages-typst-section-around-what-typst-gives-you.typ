#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the home page's Typst section around what Typst gives you",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 2),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: helps[the first screen lists what a Typst task gives you, and drops a check a JSON schema gives frontmatter too],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 12),
)

== Problem

+ Open the site root and read "Why Typst".
+ 🔴 "Values checked on load" claims a check that a JSON schema gives Markdown frontmatter too.
+ 🔴 Two editor captures, completion and then diagnostics, each sit under a paragraph of their own.


== TO-BE

- 🟢 "The elephant in the room: Typst vs Markdown + frontmatter" lists three gains: autocompletion, shared helpers and custom validators.
- 🟢 One editor capture shows tinymist's completion menu and its squiggle under `titel:` in the same file.


== Source of truth

- What tinymist returns on `examples/lsp`, recorded by `www/lsp-capture.py`.


== Changes

- `www/lsp-capture.py`: ask for completion and diagnostics on one file, and write one record.
- `www/utils/ui.typ`: one figure that draws the menu and the squiggles.
- `examples/lsp/invalid/misspelled-field.typ`: the cursor line moves here.
- `www/content/index.typ`: the section, in the owner's words.


== Out of scope

- Checks that a schema can't express: the owner decides how to show them.


== Dogfood

Not applicable: the change is the site's home page, which this repo doesn't read to work.

