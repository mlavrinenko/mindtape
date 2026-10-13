#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Raise the sweep's bars from the first batch's lane reports",
  kind: "dev",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (src-files: 3),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: neutral,
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[the sweep files work worth doing],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: done(2026, 10, 13),
)

== Problem

+ Run the first sweep batch through seven agent lanes.
+ 🔴 Lanes called three kinds of finding busywork: test modules at 90..95% of their cap, a file 3 uncovered lines below the floor, and one assert pattern filed as five dry tasks.

== TO-BE

- 🟢 The size arm reports a test module only from 97% of its cap.
- 🟢 The coverage arm skips a file with fewer than 5 uncovered lines.
- 🟢 The dry arm files one task per anchor file.

== Changes

- `contrib/ask-refactoring/size.sh`: the 97% test bar.
- `contrib/ask-refactoring/coverage.sh`: the 5-line floor.
- `contrib/ask-refactoring/dry.sh`: fold families by anchor file.

== Rollback

- Revert the commit.

== Out of scope

- The thresholds the recipe exposes; these are fixed by evidence, not per run.

== Dogfood

This repo's next sweep files fewer, larger tasks.
