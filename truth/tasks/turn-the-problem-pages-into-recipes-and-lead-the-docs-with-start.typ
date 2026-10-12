#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Turn the problem pages into recipes and lead the docs with Start",
  kind: "docs",
  priority: ice(
    impact: 6,
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
      cheap-adoption: helps[a newcomer meets Start first, then reads each recipe's effect in one table row],
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

+ Open the docs site.
+ 🔴 The sidebar opens with Problems above Start, and its entry "Split oversized tasks" opens a page titled "Tasks too big to finish in one change".
+ Open `README.md`.
+ 🔴 "What it solves" gives each problem a heading over a fenced error block, six times over.


== TO-BE

- 🟢 The sidebar opens with Start, then Recipes, and each recipe page's title is its sidebar label.
- 🟢 `README.md` and the site root list the recipes in one table under "Flexible remedy": the recipe, what it does, and the error `mt` prints when work breaks it.
- 🟢 Each recipe page reads Config, Behaviour, Deep dive.


== Source of truth

- The owner's docs review: Recipes, Start first, one name per recipe, the table and its lead-in, and the section names.
- `www/generated/<recipe>.json`: the captured sessions the table quotes.


== Changes

- `www/utils/site.typ`: Start first, the Problems group renamed Recipes.
- `www/utils/demo.typ`: the recipe table under "Flexible remedy".
- `www/utils/problems.typ`: renamed `www/utils/recipes.typ`, with `recipe-config`.
- `www/content/split-oversized-tasks.typ` and the five other recipe pages: the title is the label, the sections are Config, Behaviour and Deep dive.
- `docs/src/readme.typ`: the renamed imports.
- `outdatty.yaml`: the api-service group renamed `api-service-recipes`.
- `www/generate-api-service.sh`, `www/generate-examples.sh`: comments say recipe.


== Out of scope

- What the recipes show: progressive caps, the rule pack written out, a replacement for gate-done and a goals recipe wait on the owner's call.
- The milestone cap in the work-in-progress recipe, which has its own task.


== Dogfood

`README.md` and the site are this repo's own entrance, so the change is what this repo shows first.

