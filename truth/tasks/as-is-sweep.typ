#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rename Problem to AS-IS in the task corpus",
  priority: ice(
    impact: 5,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("mt", "stateless"),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: helps[one heading name across the backlog],
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
  status: done(2026, 11, 2),
  links: depends-on("task-shape-docs.typ"),
)


== AS-IS

+ Run `rg --hidden -l '^== Problem$' truth/tasks examples` and count the files.
+ 🔴 The backlog still uses `Problem` while the pack accepts `AS-IS`.
+ 🔴 The two headings split every search and every agent's reading of the corpus.

== TO-BE

- 🟢 Every current-shape task in `truth/tasks` and the examples uses `== AS-IS`.
- 🟢 A fence-aware rewrite leaves a quoted `== Problem` inside a code block alone, and `rg --hidden` reaches the example templates.
- 🟢 `mt check` stays green over the renamed files.

== Changes

- `truth/tasks/*.typ`: the renamed heading in the current-shape files, open tasks included.
- `examples/*/tasks/*.typ` and `examples/*/.mindtape/templates/*.typ`: the same heading in the fixtures.

== Out of scope

- The pack allowlist in `drop-problem-heading`, the docs in `task-shape-docs` and the templates in `task-templates`.

== Dogfood

Not applicable: a mechanical rename changes no feature this repo uses.
