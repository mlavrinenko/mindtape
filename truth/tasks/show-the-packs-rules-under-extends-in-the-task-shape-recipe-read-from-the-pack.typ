#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show the pack's rules under extends in the task-shape recipe, read from the pack",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 4),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader sees what one extends line brings before adopting it],
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

+ Open the "Require a task shape" recipe.
+ 🔴 Its config is `extends = ["mindtape-tasks"]`, and nothing on the page shows what the pack holds.


== TO-BE

- 🟢 Under the `extends` line, commented out, the config block shows three of the pack's rules and how many more it holds.
- 🟢 The build cuts those rules from `crates/mindtape/src/packs/mindtape-tasks.toml`, so the page can't drift from the pack.


== Source of truth

- `crates/mindtape/src/packs/mindtape-tasks.toml`: the pack.


== Changes

- `crates/mindtape/src/packs/mindtape-tasks.toml`: slice markers around three rules.
- `www/generate-examples.sh`: cut the pack's slices.
- `www/utils/recipes.typ`: render the excerpt as comments under the slice.
- `www/content/require-a-task-shape.typ`: the prose.


== Out of scope

- The Rule packs page, which lists every pack.


== Dogfood

Not applicable, this repo reads its pack in the source.
