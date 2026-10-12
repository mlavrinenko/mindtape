#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Show a milestone beside one task in the work-in-progress recipe",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the recipe shows a cap that counts a milestone apart from the task under it, as this repo's own does],
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
  status: done(
    2026,
    10,
    12,
  )[the recipe caps one milestone and one task, and its session shows both in progress],
)

== Problem

+ Open the recipe "One task at a time".
+ 🔴 Its label opens with no verb, unlike the other five.
+ 🔴 Its config caps every task at one in `wip`, so a milestone that stays open across sessions would block every task under it.


== TO-BE

- 🟢 The recipe is "Limit work in progress", and its config allows one milestone and one task in `wip`, as this repo's own config does.
- 🟢 Its session shows a milestone and a task in progress, and `mt wip` refusing a second task.


== Source of truth

- `.mindtape/config.toml`: `milestone-wip-cap` and `leaf-wip-cap`.


== Changes

- `examples/api-service/.mindtape/config.toml`: the two caps, in the renamed slice.
- `examples/api-service/tasks/`: a milestone in `wip` over the session-token work.
- `www/generate-api-service.sh`: the session.
- `www/content/one-task-at-a-time.typ`: renamed `limit-work-in-progress.typ`.
- `www/utils/site.typ`: the route.
- `skills/mindtape/evals/`, `crates/mindtape-cli/tests/it/eval_fixture_e2e.rs`: the fixture's counts.


== Out of scope

- The other recipes' content.


== Dogfood

This repo's own config holds the same two caps, and the recipe links it.

