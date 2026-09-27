#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Add Beads and Backlog.md to the comparison page",
  kind: "docs",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("www", "docs"),
  estimate: (files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: helps[a reader choosing an agent-era tracker finds the two closest ones compared on one rubric],
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: neutral,
      docs-match-code: helps[the neighbour list matches the trackers MindTape actually competes with in 2026],
    ),
  ),
  assignee: "agent",
  status: wip(2026, 9, 27),
)

== Problem

+ Open `www/content/comparison.typ`.
+ 🔴 The matrix compares dstask, Taskwarrior, todo.txt, Org-mode and git-bug, and none of them ships an agent integration. Beads and Backlog.md, the two trackers built for coding agents and the nearest in scope, are missing.
+ 🔴 No row covers what separates trackers today: agent integration, multi-agent coordination, hierarchy, hooks, human UI.


== TO-BE

- 🟢 Beads and Backlog.md sit in the neighbourhood list and the matrix, scored on the same neutral `cap` rubric as every other column.
- 🟢 New rows: agent integration, multi-agent coordination, hierarchy, hooks, human UI.
- 🟢 "Where MindTape leads" and "Where the others win" name what Beads and Backlog.md do better: claims and leases, MCP, the Kanban board.


== Source of truth

- `var/comparison-beads-backlog-md.md` (local, untracked): the source-cited study of Beads 1.3.0 and Backlog.md 1.53.0.
- The clones at `beads` `54dd4da` and `Backlog.md` `c310b708`.


== Changes

- `www/content/comparison.typ`: two columns, five rows, the neighbourhood bullets and both prose sections.
- `docs/src/readme.typ` / `README.md`: only if the page summary changes.


== Out of scope

- The feature gaps the study found: each is its own task.
- Star counts: the page argues maturity in words, not snapshots.


== Dogfood

Not applicable: a docs page.

