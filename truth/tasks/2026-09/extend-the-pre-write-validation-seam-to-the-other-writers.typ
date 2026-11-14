#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "extend the pre-write validation seam to the other writers",
  priority: ice(
    impact: 4,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 3,
  tags: ("cli",),
  estimate: (files: 3),
  assignee: "agent",
  status: done(2026, 9, 8),
)

== Summary

`mt add` renders into a buffer, validates the candidate at the exact path
it would occupy, and writes only if it passes (`mindtape::check_candidate`).
The other writers still write first and check after: `mt recur`'s spawn
creates a new file exactly as `add` did. A rejected occurrence
leaves the same broken artifact behind. `mt import` lands a whole batch.
`mt flip` and `mt review` rewrite a file that was valid before they touched
it, and report afterwards without restoring it.

== Scope

- \`mt recur\` spawn is the same defect, one command over: route it through
  \`check\_candidate\` and drop its \`verify\_written\` call.
- `mt flip` / `mt review` are a different shape: the file existed and was
  valid. Decide whether a rejected rewrite gets restored rather than whether
  it gets written. Make that call explicitly. This task does not settle it.
- \`mt import\` is a batch: does one bad entry refuse the whole batch, or land
  the rest? Answer before wiring anything.
