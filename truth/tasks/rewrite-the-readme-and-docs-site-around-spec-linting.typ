#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Rewrite the README and docs site around spec linting",
  kind: "docs",
  priority: ice(
    impact: 9,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 6,
  tags: ("docs", "www"),
  estimate: (files: 3),
  assignee: "agent",
  status: proposed(2026, 9, 14),
)

== Summary

The README sells a daemon-free file-based task tracker, which competes on features against every Markdown-and-frontmatter tracker and loses to the ones that have editors. The claim nothing else makes is that mt refuses a bad spec: an agent-written task whose Done-when says "all tests pass", whose title is not a verb, which grew a Progress section, or which estimates forty files, fails `mt check` and is told which field it broke. Dataview queries. It does not refuse.

== Scope

Lead the README and the site's front page with the linter claim and demote the tracker mechanics to the second section. That is `docs/src/readme.typ` (README.md is rendered from it and is not hand-edited), `www/content/index.typ`, and the `outdatty.lock` re-record for whichever group those two sit in. The positioning is the decision here; the page that shows a rejected spec end to end follows in `add-a-docs-page-showing-a-rejected-agent-spec.typ` and reuses this wording.
