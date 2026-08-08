#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "bring the sweep script under a size cap",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 4,
  tags: ("tooling",),
  estimate: (files: 4),
  links: related(
    "file-refactoring-tasks-from-quality-signals.typ",
  )[the script this is about],
  status: proposed(2026, 8, 8),
)

== Summary

`contrib/ask-refactoring.sh` is 750 lines. It went from 245 to that across three
arms in a day, and every line of the growth is the shape the design asked for —
a scoring function, an emit function and a block of reasoning per signal.

The joke writes itself: this is the script that files tasks about oversized
files, and it is the largest single source file in the repo. It gets away with
it because `.linecop.yaml` sets limits for Rust and Markdown only, so shell is
uncapped — the sweep is structurally unable to see itself.

== Scope

- Give shell a limit in `.linecop.yaml` first. Whatever number is chosen, choose
  it the way the 300-line Markdown overrides were chosen and write the reason
  next to it. Until the cap exists there is no signal here, only an opinion.
- Then split. The seam is obvious and was designed in from the start: one file
  per arm (`size`, `coverage`, `mutants`), each holding its `score_`/`signal_`/
  `emit_` trio and the comment block that justifies its numbers, with the
  driver, the argument parsing, the re-run guard and `stem_for` staying in the
  entry script. `$repo` is already resolved from `BASH_SOURCE`, so sourcing
  siblings costs one line each.
- Keep each arm's reasoning with its code. That prose is the reason the script
  is long and is the most valuable thing in it — moving it to a doc that drifts
  would be the wrong kind of shrinking.
- Watch the coupling: `contrib/ask-refactoring.sh` is a dependent in the
  `sweep-templates` outdatty group because it fills template slots by name. If
  the slot-filling moves into per-arm files, they are what the group must track,
  not the entry script.
- A cap on shell also puts `www/generate.sh` and the `.githooks/` scripts in
  scope. Check what else it would report before picking the number, so the cap
  lands as one deliberate decision rather than a surprise on the next sweep.
