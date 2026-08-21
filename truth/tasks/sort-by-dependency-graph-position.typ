#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Sort by dependency-graph position",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("cli",),
  estimate: (files: 3),
  assignee: "agent",
  status: done(2026, 8, 21),
)

== Summary

`links:` builds a real dependency graph, but no sort reads it. `--sort ready`
ranks on a boolean, so it separates ready from blocked without ordering
either group; `--sort depth` is folder nesting, not graph distance. There is
no way to list a backlog in the order it can actually be worked.

Add sorts derived from the graph — at minimum a topological order, and a
count of what a task transitively unblocks.

== Why

Scoring frameworks rank a task on its own properties, so a hard task ranks
low no matter how much waits behind it. On a design backlog this inverts the
order outright: the decisions everything else depends on are the expensive
ones, so ICE files them last. Observed on a fresh backlog where the two
tasks blocking the other three sorted to the bottom.

The information needed to fix it is already in the file. A task blocking five
others should outrank an isolated one at equal score, and that is a fact
`links:` already knows.

== Scope

- `--sort topo` — dependencies before dependents; stable and deterministic
  for independent tasks, with a declared tiebreak (probably the existing sort
  chain, so `--sort topo --sort priority:desc` reads naturally).
- A `blocks` sort key — how many tasks this one transitively unblocks. This
  is the one that reorders a backlog usefully, since it is a measure of
  leverage rather than of position.
- Decide whether these are also filter fields, so `mt ls 'blocks > 3'` works.
- Decide behaviour on a cycle. `mt check` rejects cycles, but sorting must
  not hang or panic on a tree that has not been checked yet.
- Decide whether terminal tasks count toward `blocks`. A done dependency
  blocks nothing, so probably not — but then the number changes as work
  closes, which is either the point or a surprise.

== Notes

Filed from a real backlog where the dependency links carried the true order
and the priority column contradicted it. Workaround today is to read
`mt ls ready` and ignore the score, which works only while the graph is small
enough to hold in your head.
