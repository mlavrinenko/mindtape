#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Rewrite the task x-goals blocks with `verdicts`",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 2,
  kind: "refactor",
  tags: ("mt",),
  estimate: (src-files: 0),
  x-goals: (
    product: (
      readable-tasks: helps[one x-goals form across the backlog],
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
  links: depends-on("as-is-sweep.typ", "verdicts-templates.typ"),
)


== Problem

+ Run `rg -l '^\s*x-goals:' truth/tasks` and count the files.
+ 🔴 Every block lists eight verdicts, and the backlog keeps the long form after the helper lands.
+ 🔴 A reader sees two forms for one field.

== TO-BE

- 🟢 Every current-shape task uses `verdicts` and keeps its verdicts.
- 🟢 `mt ls --json` returns the same `x-goals` for each task.
- 🟢 `mt check` stays green.

== Changes

- `truth/tasks/*.typ`: the `x-goals` block in the current-shape files, open tasks included.

== Out of scope

- The prelude in `prelude-verdicts`, the templates in `verdicts-templates` and the docs in `verdicts-docs`.

== Dogfood

Not applicable: a mechanical rewrite changes no feature this repo uses.
