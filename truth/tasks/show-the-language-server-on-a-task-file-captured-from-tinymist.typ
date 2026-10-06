#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show the language server on a task file, captured from tinymist",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: helps[the first screen shows field completion and a caught typo, which Markdown with YAML frontmatter cannot offer],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[what the page shows is what tinymist returns for the shipped prelude, captured in each build],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 6),
)

== Problem

+ Open the Overview and read "Why Typst".
+ 🔴 It claims "the Typst language server completes field names in your editor" and that "a misspelled field … fails with the file named", and shows neither.


== TO-BE

- 🟢 Beside those claims, the Overview shows what tinymist returns for a task file: the completions inside `task.with(...)`, and the diagnostic on a file that writes `titel:`.
- 🟢 Each is an editor-neutral block, a list under the cursor and a squiggle with its message, not a screenshot of one editor.
- 🟢 The build gets both over LSP from a fixture of its own, so a prelude change that loses a completion fails the build or changes the page.
- 🟢 The planted errors stay out of the fixtures other pages capture.


== Source of truth

- tinymist from the dev shell, 0.14.10 at filing.
- `lib/prelude.typ`: the parameters `task` declares.


== Changes

- `examples/lsp/`, new: a project with one task file to complete in and one with the misspelled field.
- `www/lsp-capture.*`, new: a minimal LSP client, in a language the dev shell has, that writes the completion items and diagnostics to `www/generated/`.
- `www/generate.sh`: call it.
- `www/utils/ui.typ`: the completion and diagnostic blocks.
- `www/content/index.typ` and the components stylesheet: place and style them.


== Out of scope

- Editor setup pages, one per editor.
- Hover, go-to-definition and other LSP features. Add one when a claim on the site needs it.


== Dogfood

The capture fails this repo's docs build when a prelude change breaks tinymist's completion, so the repo learns it before a user does.

