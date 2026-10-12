#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Open the README and the site root with the name, one lede and the Typst section",
  kind: "docs",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 7),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the first screen says what Mindtape is in two sentences],
      typst-over-markdown: helps[README answers "why Typst" where a newcomer first asks it],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    12,
  )[README and the site root open with the name and the lede, and render the Typst section from demo.typ],
)

== Problem

+ Open README.md.
+ 🔴 A tagline and an intro paragraph say the same thing twice, and the site root's H1 is that tagline.
+ Look for why tasks are Typst files.
+ 🔴 Only the site root says it. README has no Typst section, and the site's list names no computed values.


== TO-BE

- 🟢 README and the site root open with the name, then one lede: "File-based task tracker with one Typst file per task. It's flexible enough to bring your own rules and check every task and commit against them."
- 🟢 The Typst section lives once in `www/utils/demo.typ`. README and the site root render it, and the site adds the tinymist figure.
- 🟢 The section's list names computed fields.


== Source of truth

- The owner's docs review: the H1 is the name and the lede is their draft.
- `lib/prelude.typ`: the fields a task file can fill with a Typst expression.


== Changes

- `www/utils/site.typ`: the tagline becomes the lede.
- `www/utils/demo.typ`: drop the intro, add the Typst section.
- `www/utils/ui.typ`: the hero's H1 is the name.
- `www/content/index.typ`: render the shared section and the figure.
- `docs/src/readme.typ`: render the shared section.
- `www/templates/layout.typ`: the footer.
- `docs/src/contributing/documentation.typ`: name the Typst section among the shared content.


== Out of scope

- The crate and repo descriptions, which other registries show.


== Dogfood

Not applicable, this repo doesn't read its README to work.
