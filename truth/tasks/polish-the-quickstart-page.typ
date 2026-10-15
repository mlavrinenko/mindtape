#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Polish the Quickstart page",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 1),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a newcomer reads the Quickstart in plain words, with no legend, slot or default view to learn first],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the completion paragraph matches the line `mt install` now prints],
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 16),
)

== Problem

+ Open `www/content/quickstart.typ`, or the Quickstart page it builds.
+ 🔴 It explains the `@local` prelude directory, a completion hook per shell, slots, the default view and the legend before the reader has filed one task.


== TO-BE

- 🟢 The Quickstart says only what a newcomer needs, in the wording the product owner approved: paste the completion line, `mt add` asks for a title, `mt ls` shows open tasks, the project decides its statuses.


== Source of truth

- `www/generate-quickstart.sh`: the captured session, including what `mt install` prints.
- `crates/mindtape-cli/src/cmd/init/presets/flow/config.toml`: the moves the `flow` preset allows.


== Changes

- `www/content/quickstart.typ`: cut the `@local` paragraph, and replace the completion, `mt add`, `mt ls`, status and closing callout paragraphs.


== Out of scope

- Other docs pages. They keep their own depth.


== Dogfood

Not applicable. The change is docs prose.
