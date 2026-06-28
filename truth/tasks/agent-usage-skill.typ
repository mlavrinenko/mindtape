#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Agent skill — drive `mt` for basic task work",
  status: wip(2026, 6, 28)[minimal version landed; grow per-iteration before publish],
  priority: ice(impact: 5, confidence: 0.8, ease: 7),
  tags: ("cli", "docs"),
)

== Summary

A Claude agent skill (`skills/mindtape/SKILL.md`) teaching agents to drive the
`mt` CLI for basic task work — list/filter, file, re-status, validate — and to
author task `.typ` files via the DSL, instead of hand-rolling. Vendor-neutral
location; dogfooded in-repo through a `.claude/skills/mindtape` symlink. Internal
for now; publish later and drop the internal note then.

== Scope

- `skills/mindtape/SKILL.md`: detect a MindTape project, command cheat-sheet,
  `mt ls` filter language, `mt add` templates/slots, `mt flip`/status verbs,
  `mt check`, `mt go`/`mt rm`, DSL authoring, and a "drive the CLI, don't
  hand-edit" rule.
- Evals under `skills/mindtape/evals/`; the skill-creator workspace is gitignored
  scratch (`/skills/*-workspace`).
- outdatty: SKILL.md is a dependent of `cli-docs`, `cli-name`, and `prelude-docs`,
  so CLI or DSL drift re-flags the skill for review.

== Notes

- Iteration 1 benchmark (mid-tier model): with-skill 100% vs no-skill 67% over three
  cases; every baseline failure was hand-editing instead of using the CLI.
- Tuned the command section to stop preemptive `--help`/`--describe` and made the
  CLI-over-hand-edit rule explicit.
