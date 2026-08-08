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
  )[the arm this was split out of]
    + related(
      "sweep-undertested-functions-in-covered-files.typ",
    )[the signal that survived this one's measurement]
    + related(
      "cover-confirm-retire-and-unred-the-crap-gate.typ",
    )[the red gate this one's measurement surfaced]
    + related(
      "ratchet-the-crap-gate-against-a-baseline.typ",
    )[the use of cargo-crap this measurement does not rule out],
  status: cancelled(
    2026,
    8,
    8,
  )[measured: no CRAP band separates signal from noise in this repo],
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

== Outcome: cancelled, and why

Measured before deciding. `cargo crap --min 0 --format json` over the same lcov
`just crap` gates on, same exclusions, 1052 functions scored.

CRAP is $"CC"^2 dot (1 - "cov")^3 + "CC"$. The cubed term collapses to nothing
above roughly 90% coverage, so in a repo whose coverage gate is the workspace
total at 90%, CRAP degenerates into cyclomatic complexity for almost everything
it reports. That is not a threshold that was picked badly. It is the metric's
shape against this repo, and no choice of `--min` fixes it:

- Band 15..30 holds 15 functions. 11 of them are at or above 90% coverage,
  where CRAP equals CC to two decimals — `function` in `filter.rs` (CC 22,
  100% covered) scores 22.0, `Cli::run` (CC 22, 100%) scores 22.0. Filing those
  would be filing a complexity report that claims to be a coverage one. They
  are complex and thoroughly tested, which is the healthy state.
- The band also MISSES most of what it exists to find. Of the 16 functions that
  are genuinely complex and undertested (CC >= 5, coverage < 80%), 12 score
  below 15: `unlink_edits` (CC 10, 73%) scores 11.9, `weekday` (CC 8, 54%)
  scores 14.0. Lowering the edge to catch them admits 43 functions, nearly all
  of them well-tested and complex.

So the band is wrong in both directions at once — false positives above,
false negatives below — and there is no cut between them. An arm built to this
spec would have filed about 15 tasks, roughly 11 of which nobody should act on,
which is the exact cost this task's last bullet warned about.

The one thing above the gate is not an advisory finding at all: `confirm_retire`
scores 72.0 at 0% coverage and is failing `just crap` on `main` today. A signal
that already blocks is disqualified by the sweep's first rule, and it is filed
as its own task rather than as a sweep arm.

What survives is the shape, not the metric: a FUNCTION-shaped coverage signal
does see things the file-shaped arm cannot — an undertested function inside a
well-covered file, like `hval_kind` at 40% in an otherwise fine `model.rs`. 8 of
those 16 sit in files the coverage arm never reports. That is filed separately;
it is a different signal from CRAP and should not inherit this task's premise.
