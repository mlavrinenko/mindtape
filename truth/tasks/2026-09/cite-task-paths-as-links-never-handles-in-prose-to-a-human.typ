#import "@local/mindtape:0.3.0": *

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
  status: done(2026, 9, 8),
)

== Summary

A \`\#\` handle addresses the person holding the terminal that printed
it. An agent that repeats one in a report addresses it to a reader who
cannot resolve it. The command recomputes the ordinal on every run, so resolving it
means running \`mt ls\` again and hoping the id still names the same task.
Handles are efficient for an agent's own second call and stay fine there.
What has to stop is spilling them into prose.


== Scope

Edit \`docs/src/skill.typ\`. \`just docs-md\` generates \`skills/mindtape/SKILL.md\` from it,
and nobody edits the output by hand. Today the skill teaches the
handle first and offers the stem only as a fallback "for any second
operation". That framing is what puts handles in reports.

State the rule as absolute. An agent gets a conditional rule wrong on the
run that matters. Use handles freely for the
agent's own immediate follow-up call, and never write one into text a human
reads. Cite the task as a Markdown link whose target is the task file path,
so the reader clicks through instead of resolving anything.

Re-run \`just docs-md\` and commit the regenerated Markdown in the same
commit, so \`docs-md-check\` stays green.

