#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "clean the profraw pile before each coverage run",
  priority: ice(
    impact: 8,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("tooling", "tests"),
  estimate: (files: 3),
  links: related(
    "switch-coverage-to-cargo-llvm-cov.typ",
  )[the recipe this fixes, and where the --no-report split came from]
    + related(
      "ratchet-the-crap-gate-against-a-baseline.typ",
    )[the baseline this inflated, re-minted here]
    + related(
      "sweep-coverage-gaps-into-tasks.typ",
    )[a sweep arm that read the same inflated lcov]
    + related(
      "sweep-undertested-functions-in-covered-files.typ",
    )[the other arm reading it, per function],
  status: done(
    2026,
    8,
    9,
  )[cover cleans first; baseline re-minted against the first honest lcov],
)

== Summary

`just cover` instrumented once (`cargo llvm-cov --workspace --no-report`) and
rendered both reports from the profdata afterwards. It never cleaned. That split
is exactly the mechanism cargo-llvm-cov provides for MERGING several runs — its
own documented recipe for coverage across feature sets opens with
`cargo llvm-cov clean --workspace`, "before the first run" — so every `just
cover` since the recipe landed unioned its result with every profraw and object
file left under `target/llvm-cov-target` since the last `cargo clean`: other
feature sets, other commits, other environments.

Nothing surfaced it, because the union only ever moves coverage UP and every
gate downstream reads the same file. It surfaced when a `cargo clean` reset the
pile: the first honest run put 48 functions below `crap-baseline.json` and
`just check` went red with no source change behind it.

== Measurement

Two consecutive `just cover` runs left 461 + 461 profraw files, so a run merges
its predecessor rather than replacing it. The two lcovs differed in counter
values only — identical covered-line sets — so a single environment at a single
revision is stable, and the drift is between runs that differ.

The mechanism reproduces on demand. Adding one `cargo llvm-cov --workspace
--features rrule --no-report` to the pile and re-rendering took
`check_rrule_rule` from 57% to 100%: coverage of the `anchor: "completion"`
rejection, which sits behind an early `return` when `cfg!(feature = "rrule")` is
false and is therefore dead code in the build being scored. The committed
baseline had it at 90.9%, so a featured run was in the pile that minted it.

Feature mixing is not the only source, only the demonstrable one.
`ls_interactive::apply` stood at 84.2% in the baseline and 53.3% honestly; its
`DelLast`, `Clear` and `Refresh` arms have no test at all (the suite tests
`classify`, not `apply`, and `apply` takes a `&Tui`), so no run of this tree at
this revision can produce that number — old object files carry old coverage
mappings, and `llvm-cov export` unions them onto today's line numbers.
Denominators moved too, which is the signature: `check_rrule_rule` was scored
over 11 lines, honestly it has 7.

Across all 1052 scored functions the honest lcov moved 68 down and 16 up, and
changed no cyclomatic complexity at all — the ratchet's real subject never
moved, only its measurement.

== Decision

`just cover` runs `cargo llvm-cov clean --workspace` first. That forfeits the
incremental instrumented build and costs a full one, measured at 57s here, which
is the whole price. `--profraw-only` would be the cheap half and is wrong in the
other direction: it strips the counters but leaves the stale objects, whose
functions then have no profile records and read as 0% covered — the same phantom
uncovered lines the `crap` recipe already refuses a stale lcov over.

`crap-baseline.json` is re-minted against the first honest lcov, which is a
re-mint the recipe's own guard forbids: `crap-baseline: crap` refuses to mint
while the gate is red, and here the gate is red BECAUSE the baseline is wrong.
Two facts make it a correction rather than a bypass, and both were checked
first: the absolute gate (`--threshold 30 --min 30 --fail-above`) is green, so
nothing is actually bad, and no cyclomatic complexity moved. The recipe's
comment now carries that recovery path, since a guard with no documented way
out gets removed instead of respected.

== Scope

- `just cover` cleans the pile before instrumenting; the comment argues why the
  rebuild is the point and not an oversight.
- `crap-baseline.json` re-minted; `just crap` reports 1052 unchanged.
- `crap-baseline`'s comment documents the corrupted-baseline recovery.
- CONTRIBUTING.md carries the rebuild cost and the "never add `--no-clean`" rule
  where it describes the coverage run.

== Consequences

Coverage numbers before this commit are unreliable upward, including any
recorded in closed tasks. The sweep's coverage and undertested arms read this
lcov, so their next run reports findings that were previously hidden; that is
the arms working, not a regression. CI was never affected — its `coverage` job
runs in a fresh checkout with an empty pile, which is also why CI never caught
it.
