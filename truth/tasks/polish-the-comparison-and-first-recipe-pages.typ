#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Polish the comparison and first recipe pages",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("www",),
  estimate: (src-files: 11),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[each page states only what the captures and the packs show],
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 18),
)

== Problem

+ Open `www/content/comparison.typ` and the recipe pages `split-oversized-tasks`, `require-a-task-shape`, `tie-work-to-goals`, `limit-work-in-progress` and `start-titles-with-a-verb`, then `concepts.typ`.
+ 🔴 The text overstates or misstates what `mt` does. git-bug reads as exporting to Launchpad, a recipe says `mt add` refuses an empty slot, the comparison dates Mindtape's first release, and `exit-code()` can quote only the last refusal of a capture whose steps exit 2 and then 1.


== TO-BE

- 🟢 Each page carries the owner-approved wording, and every exit code it states comes from a capture.
- 🟢 `exit-code()` takes a step, and fails the build when that step is missing or exited 0.


== Source of truth

- `www/generated/require-a-task-shape.json`: the exit code of each step.
- `crates/mindtape/src/packs/mindtape-tasks.toml`: the section a `fix` adds.
- `examples/api-service/.mindtape/config.toml`: `unblocked = true` on `done`.


== Changes

- `www/utils/ui.typ`: an optional `step` argument on `exit-code()`.
- `www/content/comparison.typ`: the approved wording, and no first-release date.
- `www/generate.sh`: drop `first-release.txt`.
- `www/content/split-oversized-tasks.typ`, `www/content/require-a-task-shape.typ`, `www/content/tie-work-to-goals.typ`, `www/content/limit-work-in-progress.typ`, `www/content/start-titles-with-a-verb.typ`, `www/content/concepts.typ`: the approved wording.
- `docs/src/contributing/documentation.typ`: name the `step` argument.


== Out of scope

- `www/content/read-many-projects.typ` and `www/content/filters.typ`. Other work owns them.


== Dogfood

Not applicable. The change is docs.

