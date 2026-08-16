#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "de-slop the agent-facing docs",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("docs",),
  estimate: (files: 2),
  links: parent("de-slop-the-docs-and-close-the-drift-seams.typ")
    + depends-on("derive-the-docs-page-manifest-from-one-source.typ"),
  status: done(2026, 8, 16),
)

== Summary

`docs/src/agents.typ` renders AGENTS.md and `docs/src/skill.typ` renders
`skills/mindtape/SKILL.md`. Both are pulled into a context window in full,
every session, so a wasted line here is paid for on every run — they are the
two files where concision is measurable rather than aesthetic. Both also
carry absolute site URLs that nothing verifies.

Tighten them and move every site link onto `www-link`.

== Scope

`docs/src/agents.typ` and `docs/src/skill.typ`.

== Rules

An agent reads these as instructions, so precision beats brevity wherever the
two conflict — cut words, never a constraint. Keep every rule, every command
shape, every field name, and the whole "where validation lives" seam. What
goes: prose that motivates a rule an agent will follow anyway, the second
statement of a rule, and any pointer to a doc page that duplicates the page
instead of linking it. This task does not touch `docs/src/readme.typ`,
`contributing.typ` or `lib.typ`.
