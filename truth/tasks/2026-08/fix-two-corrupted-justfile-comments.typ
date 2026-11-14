#import "@local/mindtape:0.3.0": *

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
  status: done(
    2026,
    8,
    14,
  )[both lines restored verbatim from ddbeef5 and c373a9c, blank line added before the \[doc\] attribute so `--fmt` cannot re-truncate],
)

== Summary

Two comment defects landed in commit c373a9c and are still committed.

== The two defects

Both are in the root `Justfile`, both verified against `git show c373a9c -- Justfile`.

+ The `mmz` recipe's comment reads `` `j ust mmz fmt-check` ``, with a space
  inserted into `just`. The Justfile has no recipe-mention scan, so no gate
  wanted this and no gate caught it. Restore `` `just mmz fmt-check` ``.
+ The `check-windows` comment ends mid-sentence: `# ... so the summary is`. The
  commit that added it (ddbeef5) wrote two lines. c373a9c deleted the second.
  Restore it: `# repeated as an explicit [doc] rather than left as a dangling fragment.`

== Beyond the typo

The second is the `just --fmt` truncation this project already knows about. It
drops the last line of a comment block that has no blank line before an
attribute. Last milestone's report says review caught it. This instance proves
review did not catch it everywhere. A comment sweep that checks deletions at
one boundary does not see a truncation at another.

== Scope

- Restore both lines verbatim from git history, not from memory.
- Confirm `just --fmt --check` stays green afterwards. Add the blank line the
  formatter expects rather than re-introducing the truncation.
- Leave every other comment alone in this task.
