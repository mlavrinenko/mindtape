#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Open README and the home page on the api-service task",
  kind: "docs",
  priority: ice(
    impact: 9,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("docs", "www"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[the first screen tells an engineer which of their problems MindTape solves],
      typst-over-markdown: helps[the first screen shows a typed task and `mt` enforcing it, with no strawman beside it],
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 2),
  links: depends-on(
    "add-a-docs-page-per-problem-mindtape-solves.typ",
    "sort-the-readme-doc-index-into-the-site-nav-sections.typ",
    "write-plain-prose-rules-for-the-docs-and-check-for-history-and-shouting.typ",
  )
    + parent("make-the-docs-plain-problem-led-and-unable-to-drift.typ"),
)

== Problem

+ Read `README.md` from the top.
+ 🔴 The first code block is 40 lines of YAML that MindTape never reads, written to look worse than it is. The real task after it is this repo's own backlog task, with eight goal verdicts, `#import "../lib.typ"` and a Dogfood section.
+ 🔴 The tagline reads "A MIT-licensed". The tagline and the first sentence both open on what MindTape lacks (daemon, database).
+ The site home (`www/content/index.typ`) shares the block; its "Why Typst" bullets (plain text, no lock-in) apply to Markdown too.


== TO-BE

- 🟢 README opens with one sentence on what MindTape does for a software team, then the api-service showcase task, then one line per problem with the capture that proves it, each linking its problem page.
- 🟢 No YAML comparison anywhere. `www/readme-task.yaml`, the `readme-task.json` fact and its drift check are gone.
- 🟢 The licence moves to a License section at the end.
- 🟢 "Why Typst" lists what Typst adds over Markdown: field completion, values checked on load, shared helpers by `#import`, a Typst function as a validator.
- 🟢 Header comments in the touched sources are 10 lines or fewer, in present tense.


== Source of truth

- `examples/api-service` and its captures.
- The problem pages, for the links.


== Changes

- `docs/src/readme.typ`, `www/content/index.typ`: rewrite.
- `www/utils/demo.typ`: render the api-service slices and captures.
- `www/readme-task.yaml`: delete.
- `www/generate-facts.sh`: drop the `readme-task.json` section.
- `outdatty.yaml`: drop the twin's coupling, if any.


== Out of scope

- The doc index at the end of README: the NAV-group task builds it.


== Dogfood

This repo's README is the first page a visitor reads.

