#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "cite task paths as links never handles in prose to a human",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 2,
  tags: ("docs",),
  estimate: (files: 2),
  assignee: "agent",
  status: proposed(2026, 9, 8),
)

== Summary

A \`\#\` handle is addressed to the person holding the terminal that printed
it. An agent that repeats one in a report addresses it to a reader who
cannot resolve it: the ordinal is recomputed per command, so resolving it
means running \`mt ls\` again and hoping the id still names the same task.
Handles are efficient for an agent's own second call and stay fine there.
What has to stop is spilling them into prose.


== Scope

Edit \`docs/src/skill.typ\` — \`skills/mindtape/SKILL.md\` is generated from it
by \`just docs-md\` and must not be hand-edited. Today the skill teaches the
handle first and offers the stem only as a fallback "for any second
operation"; that framing is what puts handles in reports.

State the rule as absolute, because a conditional one is a judgment call an
agent will get wrong on the run that matters: use handles freely for the
agent's own immediate follow-up call, and never write one into text a human
reads. Cite the task as a Markdown link whose target is the task file path,
so the reader clicks through instead of resolving anything.

Re-run \`just docs-md\` and commit the regenerated Markdown in the same
commit, so \`docs-md-check\` stays green.

