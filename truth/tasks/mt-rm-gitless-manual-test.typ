#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "mt rm gitless manual test",
  priority: ice(
    impact: 4,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("cli", "tests"),
  estimate: (files: 1),
  status: accepted(2026, 8, 5)[],
  links: related(
    "mt-rm-must-not-require-git-for-a-task-nothing-references.typ",
  )[
    the implementation this session verifies
  ],
)

== Summary

Verify mt rm removes an unreferenced task in a real session outside a git repo, and rewires links when a survivor references the removed task.

== Scope

- Scratch project with a tasks root outside any git repo: `mt add` a task, mark
  it done, then `mt rm` it — no git involvement, no error mentioning a
  repository, file gone, `mt check` green.
- Same project inside a git repo: unreferenced task still deletes directly.
- Scratch project where a survivor links to the removed task: `mt rm` rewires
  the edge to the tombstone as before (git path unchanged).
- `mt rm --dry-run` outside a git repo: prints the deletion plan and writes
  nothing.
