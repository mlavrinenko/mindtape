#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rewrite the README and docs site around spec linting",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("docs", "www"),
  estimate: (files: 8),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

The README sells a daemon-free file-based task tracker, which competes on features against every Markdown-and-frontmatter tracker and loses to the ones that have editors. The claim nothing else makes is that mt refuses a bad spec: an agent-written task whose Done-when says "all tests pass", whose title is not a verb, which grew a Progress section, or which estimates forty files, fails `mt check` and is told which field it broke. Dataview queries. It does not refuse.

== Scope

Lead README.md and www/content/index.typ with the linter claim and demote the tracker mechanics to the second section. Give the docs site one page whose entire content is a rejected agent-written task and the mt check output that rejected it. The card render stops being a selling point and becomes a footnote.
