#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "sweep coverage gaps into tasks",
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

`just cover` gates at 90% lines and `just crap` fails any function above CRAP
30, and CI enforces both. So a breach already blocks — and by this sweep's own
rule a blocking signal can never become a task, because the wall is already in
front of you. The advisory signal is the band *below* each gate: a function
climbing toward CRAP 30, a file drifting toward 90%. That is the same shape as
`linecop --baseline 85`, which is the sweep's first signal, so the two read
alike and neither needs stored state.

== Scope

- File on an advisory band against the current measurement, never on a delta
  against a stored baseline. A recorded prior-run JSON was the obvious design
  and is the wrong one: it churns on every coverage run, conflicts between
  contributors editing different code, and is absent on a fresh clone — where
  its absence silently disables the signal rather than announcing itself.
- Pick both band edges once from the real distribution, and record the numbers
  in the recipe with the reasoning, so they are not re-litigated per run. A
  band that catches nearly a thousand functions is noise; one that catches
  nothing is theatre.
- The lcov is a build artifact nothing re-derives, and `just crap` already
  refuses one older than the sources, because a file the lcov never saw scores
  as 0% covered rather than as unknown. The sweep must run `just cover` first
  or honour the same refusal — it must never file a phantom finding.
- One task per function, naming the uncovered lines, which
  `llvm-cov --show-missing-lines` reports.
- Memoize `just cover` with mmz so a re-sweep of an unchanged tree costs
  nothing.
