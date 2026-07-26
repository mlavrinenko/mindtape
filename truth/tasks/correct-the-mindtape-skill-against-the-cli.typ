#import "@local/mindtape:0.2.0": *

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
  )[skill verified command-by-command against the installed mt; three traps covered by evals],
)

== Summary

A session driving MindTape through the agent skill hit three inaccuracies: the skill documented an mt ls --all flag the CLI rejects, it did not warn that an unrecognised mt add --set key is dropped silently with exit 0, and its status guidance assumed a legend that may declare blocked when blockage is really a depends-on edge. Correct the skill against the real CLI and cover the traps with evals.

== Scope

- Verify every documented command and flag against the installed mt before writing.
- Replace the --all claim, document the silent --set drop and framework-leaf slot names, and add a section on modelling blockage as a links: depends-on edge (including the external-blocker case when the legend has no blocked value).
- Add evals for the three traps; keep SKILL.md within the 200-line linecop cap.
