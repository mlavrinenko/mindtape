#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "fix two corrupted justfile comments",
  priority: ice(
    impact: 3,
    confidence: 0.95,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("tooling",),
  estimate: (files: 1),
  status: wip(2026, 8, 14)[wave 1, delegated],
)

== Summary

Two comment defects landed in commit c373a9c and are still committed.

== The two defects

Both are in the root `Justfile`, both verified against `git show c373a9c -- Justfile`.

+ The `mmz` recipe's comment reads `` `j ust mmz fmt-check` `` — a space inserted
  into `just`. Nothing scans the Justfile for recipe mentions, so no gate wanted
  this and no gate caught it. Restore `` `just mmz fmt-check` ``.
+ The `check-windows` comment ends mid-sentence: `# ... so the summary is`. The
  commit that added it (ddbeef5) wrote two lines; c373a9c deleted the second.
  Restore it: `# repeated as an explicit [doc] rather than left as a dangling fragment.`

== Why it matters beyond the typo

The second is the `just --fmt` truncation this project already knows about — it
drops the last line of a comment block that has no blank line before an
attribute. Last milestone's report says it was caught in review; this instance
proves it was not caught everywhere. A comment sweep that checks deletions at
one boundary does not see a truncation at another.

== Scope

- Restore both lines verbatim from git history, not from memory.
- Confirm `just --fmt --check` stays green afterwards: add the blank line the
  formatter wants rather than re-introducing the truncation.
- No other comment changes in this task.
