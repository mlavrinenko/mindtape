#import "@local/mindtape:0.2.0": *

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
  status: proposed(2026, 9, 7),
)

== Summary

\`mt add\` now renders into a buffer, validates the candidate at the exact path
it would occupy, and writes only if it passes (\`mindtape::check\_candidate\`).
The other writers still write first and check after: \`mt recur\`'s spawn
creates a NEW file exactly the way \`add\` used to, so a rejected occurrence
leaves the same broken artifact behind; \`mt import\` lands a whole batch;
\`mt flip\` and \`mt review\` rewrite a file that was valid before they touched
it, and report afterwards without restoring it.

== Scope

- \`mt recur\` spawn is the same defect, one command over: route it through
  \`check\_candidate\` and drop its \`verify\_written\` call.
- \`mt flip\` / \`mt review\` are a different shape — the file existed and was
  valid, so the question is whether a rejected rewrite is restored rather
  than whether it is written. Decide that explicitly; it is not settled here.
- \`mt import\` is a batch: does one bad entry refuse the whole batch, or land
  the rest? Answer before wiring anything.
