#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "adopt a files estimate on the open backlog",
  priority: ice(
    impact: 6,
    confidence: 0.7,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("mt", "gating"),
  estimate: (files: 5),
  status: done(2026, 7, 23),
  links: related("files-budget-drift-check.typ")[
    Split off from it. That task shipped the feature and put `units` and the
    caps on this repository's config; requiring an estimate needs a backfill of
    every open task, which is the part left here.
  ],
)

== Summary

Declare a files estimate on every open task, then set required on \[tasks.estimate\], finishing the adoption files-budget-drift-check shipped the machinery for.

The feature landed with units = \["files"\] and max/min caps on this repository's config, but required is still unset, so nothing forces a task to declare a budget. Until it does, the drift warning has nothing to compare against on most tasks and the check is dormant.

== Scope

- Give every non-terminal task an estimate: (files: N) reflecting the files it means to touch. Terminal tasks are exempt by construction and must not be backfilled — a closed task cannot be split, and scoring finished work invents a prediction nobody will act on.
- Only then set required = "!terminal" on \[tasks.estimate\], mirroring priority and difficulty. Flipping it first fails the whole backlog at once.
- Add an estimate slot to .mindtape/templates/task.typ so mt add asks for it, rather than writing a task that fails validation the moment it lands.
- Sanity-check the cap while backfilling: the config allows 1 to 5 files. A task that genuinely needs more is the split signal the budget exists to raise, not a reason to raise the cap.
- Tests: the dogfood config validates with required set; mt add with no explicit estimate still produces a valid task; a task over the cap fails validation.
