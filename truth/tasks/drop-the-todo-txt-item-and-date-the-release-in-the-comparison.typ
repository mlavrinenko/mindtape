#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Drop the todo.txt item and date the release in the comparison",
  kind: "docs",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 7,
  ),
  difficulty: 1,
  tags: ("www", "docs"),
  estimate: (src-files: 2),
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
      docs-match-code: helps[the maturity item reads its date from CHANGELOG.md and cannot go stale],
    ),
  ),
  assignee: "agent",
  status: done(
    2026,
    10,
    12,
  )[the todo.txt item is gone, and the maturity item states the first release date from CHANGELOG.md],
)

== Problem

+ Open the comparison page and read "When another tool fits better".
+ 🔴 "The least tooling" says todo.txt is a text file any editor opens, which a Typst task file is too.
+ 🔴 "Maturity" says Mindtape has fewer users, and it has none outside this repo yet.


== TO-BE

- 🟢 The todo.txt item is gone.
- 🟢 The maturity item says when Mindtape was first released, read from `CHANGELOG.md`.


== Source of truth

- `CHANGELOG.md`: the dated release headings.


== Changes

- `www/content/comparison.typ`: the two items.
- `www/generate.sh`: the first release date as a generated fact, because the page can't read `CHANGELOG.md` from under `www/`.


== Out of scope

- The matrix rows.


== Dogfood

Not applicable, the comparison is for readers choosing a tracker.
