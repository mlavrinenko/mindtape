#import "@local/mindtape:0.3.0": *
#import "/truth/lib.typ": *

#show: task.with(
  title: "Open the README and the site root with the name, one intro and the Typst section",
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
      cheap-adoption: helps[the first screen says what the tool is in two sentences.],
      typst-over-markdown: helps[README answers "why Typst" on the page a reader opens first.],
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
  )[README and the site root open with the name and the intro, and render the Typst section from demo.typ],
)

== AS-IS

+ Open README.md.
+ 🔴 A tagline and an intro paragraph say the same thing twice, and the site root's H1 is that tagline.
+ Look for why tasks are Typst files.
+ 🔴 Only the site root says it. README has no Typst section, and the site's list does not show a computed value.


== TO-BE

- 🟢 README and the site root open with the name, then one intro: "File-based task tracker with one Typst file per task. It's flexible enough to bring your own rules and check every task and commit against them."
- 🟢 The Typst section lives once in `www/utils/demo.typ`. README and the site root render it, and the site adds the tinymist figure.
- 🟢 The section's list names computed fields.


== Source of truth

- The owner's docs review sets the H1 as the name and the intro as their draft.
- `lib/prelude.typ`: the fields a task file can fill with a Typst expression.


== Changes

- `www/utils/site.typ`: the tagline becomes the intro.
- `www/utils/demo.typ`: drop the intro, add the Typst section.
- `www/utils/ui.typ`: the hero's H1 is the name.
- `www/content/index.typ`: render the shared section and the figure.
- `docs/src/readme.typ`: render the shared section.
- `www/templates/layout.typ`: the footer.
- `docs/src/contributing/documentation.typ`: list the Typst section among the shared content.


== Out of scope

- The crate and repo descriptions, which other registries show.


== Dogfood

Not applicable, this repo doesn't read its README to work.
