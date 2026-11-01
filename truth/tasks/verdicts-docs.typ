#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Document the `verdicts` helper and the short x-goals form",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("docs", "www"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: helps[the docs show the form the template files],
      implementable-tasks: neutral,
      enforced-practice: neutral,
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
  status: proposed(2026, 11, 1),
  links: depends-on("verdicts-templates.typ")
    + depends-on("task-shape-docs.typ"),
)



== Problem

+ Read `www/content/tie-work-to-goals.typ` and the skill's task example.
+ 🔴 The goals page shows the `goals.typ` helper and the skill has no `x-goals` example, so the short form is absent from the docs.
+ 🔴 The prelude's new helpers are absent from the docs.

== TO-BE

- 🟢 The goals page shows the short form and explains the merge.
- 🟢 The skill tells an agent to list the neutral goals explicitly.
- 🟢 The package version in the docs matches `lib/typst.toml`.

== Source of truth

- `lib/prelude.typ` and `truth/lib.typ`: the helpers and the check.
- `lib/typst.toml`: the version the docs quote.

== Changes

- `www/content/tie-work-to-goals.typ`: the short form.
- `docs/src/skill.typ`: the filing step and the helper.
- `examples/api-service/goals.typ` and `examples/api-service/.mindtape/templates/milestone.typ`: the fixture that shows the short form.

== Out of scope

- The prelude in `prelude-verdicts`, the templates in `verdicts-templates` and the corpus in `x-goals-sweep`.

== Dogfood

The docs site is where an adopter meets the helpers.
