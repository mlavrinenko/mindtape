#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "Correct the mindtape skill against the CLI",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("docs",),
  estimate: (files: 2),
  status: done(
    2026,
    7,
    26,
  )[skill verified command-by-command against the installed mt. Three traps covered by evals],
)

== Summary

A session driving the tool through the agent skill hit three inaccuracies. The skill documented an `mt ls --all` flag the CLI rejects, and it did not warn that an unrecognised `mt add --set` key disappears silently with exit 0. Its status guidance also assumed a legend that may declare blocked, when blockage is really a depends-on edge. Correct the skill against the real CLI and cover the traps with evals.

== Scope

- Verify every documented command and flag against the installed mt before writing.
- Replace the `--all` claim and document the silent `--set` drop and framework-leaf slot names. Add a section on modelling blockage as a links: depends-on edge, including the external-blocker case when the legend has no blocked value.
- Add evals for the three traps. Keep SKILL.md within the 200-line linecop cap.
