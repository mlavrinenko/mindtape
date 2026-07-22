#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "external artifact links",
  priority: ice(
    impact: 7,
    confidence: 0.7,
    ease: 5,
  ),
  tags: (),
  status: proposed(2026, 7, 22),
)

== Summary

Let a task link something outside the backlog — the commit, PR or build that discharges it. Today every `links:` target resolves to a task file stem, so there is no way to record what a done stamp is actually backed by. That gap is what keeps a review gate a vocabulary rather than an enforcement point: MindTape can say a task reached done, but not that anything was produced. It is also the missing half of the done gate, which needs an artifact to check.

== Scope

- `[[tasks.links.kinds]]` gains `external = true`. Targets of an external kind are opaque strings — a URL, a commit sha, a PR ref — not resolved as stems.
- External edges are excluded from resolution, cardinality-against-tasks, cycle detection and readiness. Note rules still apply, so a kind can demand a note.
- Validation instead checks target shape against an optional per-kind `pattern` regex, so a commit kind can reject prose.
- `mt ls` and `mt graph` render external targets distinctly and never try to follow them.
- Tests: external target does not fail stem resolution, participates in no cycle, pattern rejects a malformed target, note rule still enforced, graph render omits them from traversal.
