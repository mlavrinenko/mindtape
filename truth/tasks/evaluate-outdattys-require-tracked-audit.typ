#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "evaluate outdattys require tracked audit",
  priority: ice(
    impact: 5,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("tooling",),
  estimate: (files: 3),
  status: proposed(2026, 8, 15),
)

== Summary

outdatty 0.4 added \`require\_tracked\` — an audit that every non-gitignored file belongs to some group — and defaults it to \`\["\*\*"\]\`. The bump opted out with \`\["!\*\*"\]\` rather than absorb a new default silently. This task decides whether any scope of it is worth turning back on.

== Scope

- Measure first: \`outdatty check\` lists every uncovered file. 581 today — 276 crate sources, 235 task artifacts, and the rest split across www/, docs/, contrib/.
- The audit fits one failure this repo has already had: a new source or doc page that no group tracks, so it rots with nothing reacting. \`cli-docs\` names its command modules one by one for exactly that reason, and \`just check-doc-coverage\` only closes the half of the gap that is about CLI verbs.
- Decide per surface, not repo-wide. A candidate scope like \`\["!\*\*", "docs/src/\*\*"\]\` is meaningful and small; \`crates/\*\*\` and \`truth/tasks/\*\*\` are not candidates at all — a drift group means "review these dependents when this source changes", and stretching it to mean "this file is known" would make every group noise.
- Whatever is chosen, no file gets added to a group to satisfy the audit. A group that exists to answer a coverage check is worse than an uncovered file.
- If nothing scopes cleanly, record that as the answer and keep \`\["!\*\*"\]\` — the comment in outdatty.yaml then points here instead of reading as a TODO.
