#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "decide what a dependent-only edit owes outdatty.lock",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 2),
  status: proposed(2026, 8, 16),
)

== Summary

`outdatty check` verifies dependents are confirmed for every changed SOURCE, so
a dependent edited on its own moves no source hash, keeps the gate green, and
leaves its recorded hash in `outdatty.lock` stale until some unrelated source
edit re-records the group. CONTRIBUTING.md calls a recorded hash "a review
watermark"; a watermark that heals on a commit that never touched the file
misattributes the review it claims to record.

== The evidence

Both halves happened in this repo, five commits apart:

- `d1f9ede` edited `docs/src/contributing.typ` and `CONTRIBUTING.md` — both
  `dev-docs` dependents — and touched no `outdatty.lock` line. `just check` was
  green, correctly: no source changed.
- `0c96610` added one attribute to the `mmz` recipe and ran `just
  outdatty-update dev-docs`. Its lock diff carries a
  `docs/src/contributing.typ` hash change the commit never made, sitting beside
  the `Justfile` hash it did.

Read from the lock alone, `contributing.typ` was reviewed as part of a
`[no-exit-message]` change. It was not.

== Scope

- Measure before deciding: run `outdatty status` against a tree with a
  hand-edited dependent and see whether it already reports the drift. If it
  does, this may be a docs change in `outdatty.yaml`'s header, nothing more.
- The three shapes worth weighing. Accept it and say so in the manifest header,
  so the next reader of a lock diff is not surprised. Or make the commit that
  edits a dependent re-record its group, which puts the watermark on the right
  commit at the cost of a step nothing enforces. Or take it upstream — outdatty
  reporting dependent-hash drift in `status` makes it visible without gating.
- No new gate that fails on a dependent edit. A dependent changing alone is
  ordinary work, not drift, and the whole point of the source/dependent split
  is that only one side is a trigger. A rule that reverses that trades a
  cosmetic lock-diff problem for a red gate on every doc edit.
- Whatever is chosen, the answer belongs in `outdatty.yaml`'s header comment
  next to the `require_tracked` note, which is already where this repo argues
  its outdatty shape.
