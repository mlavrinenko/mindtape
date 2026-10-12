#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Replace the gate-done recipe with one that ties milestones to goals",
  kind: "docs",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("www", "docs"),
  estimate: (src-files: 8),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[the docs show the goal check this repo runs on its own backlog],
      cheap-adoption: neutral,
      typst-over-markdown: helps[a Typst function checks a field no schema can],
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

+ Open the "Gate done on your checks" recipe.
+ 🔴 It runs the tests when a task closes, which a pre-commit hook already does whenever the flip rides in the work's last commit.
+ Look for goals in the docs.
+ 🔴 No page shows `x-goals` and a `check` rule, though this repo rates every open task against its goals.


== TO-BE

- 🟢 A recipe makes every open milestone in `examples/api-service` rate each goal in a goals module, and refuses one that rates none or helps none.
- 🟢 Its capture shows `mt` refusing a milestone with no goals, and the milestone that passes.
- 🟢 No page links the gate-done recipe, and Flip hooks keeps its own `stale(30)` example.


== Source of truth

- `truth/lib.typ`: `check-goals`, the dogfooded check.
- `.mindtape/config.toml`: the `goals` rule.


== Changes

- `examples/api-service/goals.typ`: the goals and `check-goals`.
- `examples/api-service/.mindtape/config.toml`: the goals slice replaces the flip gate.
- `examples/api-service/tasks/harden-session-tokens.typ`: rate the goals.
- `examples/api-service/scripts/test`: delete it, as no rule runs it.
- `www/content/tie-work-to-goals.typ`: the recipe page, in place of `gate-done-on-checks.typ`.
- `www/utils/site.typ`: the nav entry.
- `www/generate-api-service.sh`: the capture.
- `www/content/hooks.typ`: drop the link to the old recipe.


== Out of scope

- The `[[on.flip]]` docs, which Flip hooks covers.


== Dogfood

This repo's `goals` rule is the recipe's model.
