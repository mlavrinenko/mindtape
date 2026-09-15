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
  status: cancelled(
    2026,
    9,
    15,
  )[Superseded on 2026-09-15 by rewrite-the-readme-around-what-typst-tasks-express and add-a-docs-page-on-tasks-that-validate-themselves: the pitch is what Typst tasks express that Markdown with frontmatter cannot.],
  links: depends-on(
    "replace-the-task-section-set-with-a-closed-four-section-spec.typ",
  )[the pitch quotes the refusals the four-section spec adds],
)

== Summary

The README sells a daemon-free file-based task tracker, which competes on features against every Markdown-and-frontmatter tracker and loses to the ones that have editors. The claim nothing else makes is that mt refuses a bad spec: an agent-written task whose Done-when says "all tests pass", whose title is not a verb, which grew a Progress section, or which estimates forty files, fails `mt check` and is told which field it broke. Dataview queries. It does not refuse.

== Scope

Lead README.md and www/content/index.typ with the linter claim and demote the tracker mechanics to the second section. The card render stops being a selling point and becomes a footnote.

Split: the page whose entire content is a rejected agent-written task and the mt check output that rejected it is `add-a-docs-page-showing-a-rejected-agent-spec.typ`. README.md is rendered from `docs/src/readme.typ`, so that source is the file touched, with the `outdatty.lock` re-record for its group.

== Dogfood

Not applicable: docs only.
