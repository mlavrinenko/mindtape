#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "adopt nearest match suggestions at the remaining unknown name errors",
  priority: ice(
    impact: 5,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("cli",),
  estimate: (files: 6),
  assignee: "agent",
  links: depends-on("suggest-a-nearest-match-on-an-unknown-view-name.typ"),
  status: done(
    2026,
    9,
    7,
  )[landed across the merge; derived-tag site wired at integration],
)

== Summary

Follow-up to \`suggest-a-nearest-match-on-an-unknown-view-name.typ\`: once the
shared nearest-name helper exists in \`crates/mindtape\`, adopt it at every
other unknown-name error. Most of these already enumerate the legal values
but none offers a nearest match, and one enumerates nothing at all.


== Scope

- Columns: \`crates/mindtape-cli/src/ls\_rules.rs:356\`,
  \`crates/mindtape/src/config\_ls.rs:294\`, \`:429\`, \`:437\`.
- Status legend: \`crates/mindtape-cli/src/flip.rs:131\`; and the shortcut-verb
  miss at \`:106\`, where \`mt \<typo> \<task>\` reports \`unknown command\` without
  naming a near verb.
- Slot keys: \`crates/mindtape-cli/src/add\_values.rs:145\`.
- Tags: \`\[tasks.tags\] allowed\` rejects with "\`dx\` is not an allowed tag" and
  enumerates nothing — add both the suggestion and the allowed list.
- Worktree aliases: \`crates/mindtape-cli/src/worktrees.rs:129\`
  (\`unknown\_worktree\`) already lists known aliases and the config path; add
  the nearest match on top and reuse the shared formatter so the shape
  matches the view error.
- Note the collision worth documenting while here: a leading \`\@token\` is
  worktree-alias sugar rewritten by \`worktrees::desugar\_argv\`
  (\`crates/mindtape-cli/src/main.rs:88\`) before clap, so \`mt \@agent\` can
  never mean \`\[views.agent\]\` — views are referenced in argument position
  only (\`mt ls \@agent\`). Confirm the two error messages are distinguishable
  enough that a user who confused the two can tell which namespace missed.
- Each site keeps its existing enumeration; the suggestion line is added,
  not substituted.

