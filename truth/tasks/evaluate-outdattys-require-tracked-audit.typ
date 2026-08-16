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
  status: done(2026, 8, 16),
)

== Summary

outdatty 0.4 added \`require\_tracked\` — an audit that every non-gitignored file belongs to some group — and defaults it to \`\["\*\*"\]\`. The bump opted out with \`\["!\*\*"\]\` rather than absorb a new default silently. This task decides whether any scope of it is worth turning back on.

== Scope

- Measure first: \`outdatty check\` lists every uncovered file. 581 today — 276 crate sources, 235 task artifacts, and the rest split across www/, docs/, contrib/.
- The audit fits one failure this repo has already had: a new source or doc page that no group tracks, so it rots with nothing reacting. \`cli-docs\` names its command modules one by one for exactly that reason, and \`just check-doc-coverage\` only closes the half of the gap that is about CLI verbs.
- Decide per surface, not repo-wide. A candidate scope like \`\["!\*\*", "docs/src/\*\*"\]\` is meaningful and small; \`crates/\*\*\` and \`truth/tasks/\*\*\` are not candidates at all — a drift group means "review these dependents when this source changes", and stretching it to mean "this file is known" would make every group noise.
- Whatever is chosen, no file gets added to a group to satisfy the audit. A group that exists to answer a coverage check is worse than an uncovered file.
- If nothing scopes cleanly, record that as the answer and keep \`\["!\*\*"\]\` — the comment in outdatty.yaml then points here instead of reading as a TODO.

== Decision

Measured per surface with a scratch edit of the manifest, reverted after: the audit hard-fails \`outdatty check\` (exit 1, an \`[untracked]\` section naming every in-scope file covered by no group) the moment any in-scope file is uncovered — a red \`just check\`, not a listing. Scopes tried, uncovered files each: \`\["!\*\*", "docs\/src\/\*\*"\]\` — 2 (docs/src/contributing/generated-docs.typ, docs/src/lib.typ); \`\["!\*\*", "www\/content\/\*\*"\]\` — 2 (comparison.typ, internals.typ); \`\["!\*\*", "www\/utils\/\*\*"\]\` — 9 (page.typ, tola.typ, ui.typ and the six \*-notes.typ/\*-ref.typ helpers); \`\["!\*\*", "skills\/mindtape\/\*\*"\]\` — 1 (SKILL.md). Every narrow doc surface is dirty.

Keep \`\["!\*\*"\]\`.

The manifest comment's claim holds up: no narrow scope is free, and the uncovered files are deliberate, not gaps — lib.typ documents itself as "NOT itself a source", SKILL.md is a generated output whose tracked source is already the group member, and the www helpers are library modules no source has dependents on. Turning the audit on would mean adding each to a group, and a group that exists to answer a coverage check is worse than an uncovered file: a drift group means "review these dependents when this source changes", not "this file is known", which is not a fact worth gating. The failure the audit exists for — a new page no group tracks — cannot be had at any scope without first paying that cost, so the opt-out stays and the comment above it points here instead of reading as a TODO.
