#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Cap only judgment work in the split-oversized recipe",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the recipe shows the cap this repo enforces],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
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

+ Open the "Split oversized tasks" recipe.
+ 🔴 It caps every open task at 5 source files, so a 12-file rename can't be filed, though splitting it only copies one decision into several tasks.


== TO-BE

- 🟢 The recipe caps judgment work, `difficulty >= 3`, at 5 source files and leaves mechanical work uncapped, as this repo does.
- 🟢 Its capture shows a hard 7-file task refused and an easy 12-file rename filed.


== Source of truth

- `.mindtape/config.toml`: the `judgment-stays-small` rule and its reasons.


== Changes

- `examples/api-service/.mindtape/config.toml`: the difficulty field and the capped rule.
- `www/generate-api-service.sh`: the two captures.
- `www/content/split-oversized-tasks.typ`: the prose.


== Out of scope

- The working-tree budget rule, which the Estimates page covers.


== Dogfood

The recipe mirrors `judgment-stays-small` in this repo's config.
