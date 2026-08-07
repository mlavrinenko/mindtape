#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "sweep coverage regressions into tasks",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("tooling", "tests"),
  estimate: (files: 4),
  links: depends-on("file-refactoring-tasks-from-quality-signals.typ"),
  status: proposed(2026, 8, 8),
)

== Summary

\`just cover\` gates at 90% lines and \`just crap\` fails functions above CRAP 30, both CI-only. Neither leaves anything behind for the work they imply. cargo-crap already emits \`--format json\` and accepts a \`--baseline\` from a prior run, reporting regressions, improvements and new findings — a delta, which is the only version of this signal worth filing, since the absolute list is nearly a thousand functions.

== Scope

- File on regression against a recorded baseline, not on absolute score. A function that has always been at CRAP 29 is not news.
- The lcov is a build artifact nothing re-derives, and \`just crap\` already refuses one older than the sources because a file the lcov never saw reads as 0% covered rather than as unknown. The sweep must run \`just cover\` first or honour the same refusal — it must never file a phantom finding.
- Per-function tasks, naming the uncovered lines, which \`llvm-cov --show-missing-lines\` reports.
- Store the baseline where a weekly run can find it, and state in each task which baseline it is a regression against.
