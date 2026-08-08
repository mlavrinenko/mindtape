#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "sweep CRAP scores into tasks",
  priority: ice(
    impact: 4,
    confidence: 0.6,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("tooling", "tests"),
  estimate: (files: 3),
  links: related(
    "sweep-coverage-gaps-into-tasks.typ",
  )[the arm this was split out of],
  status: proposed(2026, 8, 8),
)

== Summary

`sweep-coverage-gaps-into-tasks` argued for a band below each of the two
coverage-adjacent gates: `just cover` at 90% lines, and `just crap` at CRAP 30.
Only the first was built. The band below CRAP 30 is a genuinely different
signal and was split out here rather than folded in, because it is measured per
FUNCTION where the coverage arm is per file — a different template, a different
stem shape, and a different remedy.

It is also the more interesting of the two. A CRAP score pairs cyclomatic
complexity with coverage, so a function climbing toward 30 is complex *and*
undertested, which is a sharper claim than either number alone. Whether it is
sharp enough to be worth a task per finding is the open question, and the
reason confidence sits at 0.6.

== Scope

- One task per function, not per file. That needs a fourth
  `.mindtape/templates/sweep-*.typ` template — the existing three are the
  shared shape the `sweep-templates` outdatty group keeps in step, so a
  function-shaped one joins that group as a dependent.
- `just crap` runs with `--min 30 --threshold 30 --fail-above`, so it prints
  only the functions that already fail. The advisory band needs a lower
  `--min`; pick the edge from the real distribution the way the coverage arm
  picked 88, and record the numbers with the reasoning next to the code. The
  recipe's own comment notes 976 analysed functions, so a loose edge is noise
  by a wide margin.
- Read the same lcov the coverage arm reads, and honour the same freshness
  refusal — `ensure_lcov_fresh` already exists and `just crap` already refuses
  a stale lcov for the documented reason (a file the lcov never saw scores 0%,
  not unknown). Do not add a second staleness mechanism.
- The remedy differs from both existing arms: a high-CRAP function is usually
  split or simplified before it is tested, so the template should say that
  rather than reach for the coverage arm's "write the missing test".
- Decide honestly whether it earns a place. Closing this as cancelled with the
  reason written down is a real outcome — the sweep is only worth what its
  findings are, and an arm that files nine plausible-looking tasks nobody acts
  on costs more than it returns.
