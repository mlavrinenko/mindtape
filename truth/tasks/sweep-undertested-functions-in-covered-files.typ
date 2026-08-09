#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "sweep undertested functions inside covered files",
  priority: ice(
    impact: 5,
    confidence: 0.7,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("tooling", "tests"),
  estimate: (files: 3),
  links: related(
    "sweep-crap-scores-into-tasks.typ",
  )[the cancelled arm whose measurement produced this]
    + related(
      "sweep-coverage-gaps-into-tasks.typ",
    )[the file-shaped arm this would sit beside],
  status: done(
    2026,
    8,
    9,
  )[built: dedup by deferral to the file-shaped arm, band at CC 5 / coverage 80],
)

== Summary

`sweep-crap-scores-into-tasks` was cancelled on measurement: CRAP degenerates to
cyclomatic complexity above ~90% coverage, so a band below the CRAP-30 gate
reports well-tested complex functions and misses undertested ones. The reasoning
and the numbers are in that task.

What the measurement did establish is that the SHAPE was right even though the
metric was wrong. The coverage arm is file-shaped, and a file passes the 88%
floor while still hiding a function nobody tests: `hval_kind`
(`crates/mindtape/src/model.rs`) sits at 40% coverage inside a file the coverage
arm never reports. A file-shaped signal is structurally unable to see that.

The candidate signal is the conjunction the CRAP metric only approximates —
cyclomatic complexity at or above some floor AND line coverage below some
ceiling — read from the same lcov, per function. At CC >= 5 / coverage < 80% it
finds 16 functions today, and 8 of them are in files the coverage arm never
reports. Those 8 are the whole argument for this arm; the other 8 are already
covered by a file-shaped task and would be duplicates.

Confidence is 0.7 because that overlap is the open question, not the thresholds.

== Scope

- Decide the deduplication first, before any thresholds. Half of today's
  findings are in files the coverage arm already files a task for, and two arms
  filing about the same file is how the re-run guard ends up blocking on twice
  the work. Either this arm skips functions whose file the coverage arm reports,
  or the two are genuinely one arm that picks its granularity per finding. That
  choice decides whether this is worth building at all.
- Pick CC and coverage edges from the real distribution the way the coverage arm
  picked 88, and record the numbers with the reasoning next to the code — which
  since `bring-the-sweep-script-under-a-size-cap` means in
  `contrib/ask-refactoring/<arm>.sh`, with the arm named in `ARMS`.
- Read the same lcov, and honour `ensure_lcov_fresh` rather than adding a second
  staleness mechanism. Complexity can come from `cargo crap --format json`,
  which already reports `cyclomatic` per function alongside `coverage` — using
  it as a complexity source is not the same as banding on its CRAP score, and
  the distinction is the whole reason the previous task was cancelled.
- The remedy is the coverage arm's, not the size arm's: write the missing test,
  or exclude with a written reason. A function at CC 5 and 57% coverage is not
  asking to be split.
- One task per FUNCTION, so the stem must carry the function name. `stem_for`
  namespaces by signal already; the mutants arm's stem rule (derive from what
  identifies the finding, never a line number, so the stem survives edits above
  it) is the precedent to follow.
- Cancelling this is still a real outcome. Its predecessor was cancelled on
  data, and the same standard applies: if the deduplication question does not
  have a clean answer, this arm files noise beside an arm that already works.

== Outcome: built, and the deduplication answer that decided it

The arm ships as `contrib/ask-refactoring/undertested.sh` with
`.mindtape/templates/sweep-undertested.typ`, named in `ARMS` after `coverage`.

=== The deduplication, answered first

The two candidate answers were "skip functions whose file the coverage arm
reports" and "one arm that picks its granularity per finding". The first is
right, and not as a compromise: the remedies NEST. A file below the coverage
floor gets one task whose work is covering the functions inside it, so a
function-level task beside it is that task's first bullet with a stem of its
own — pure duplication, and under the re-run guard a duplicate pair blocks the
next sweep until both halves are closed. Above the floor, this arm is the only
reporter, so nothing is dropped either. The skip is exact rather than heuristic:
`coverage_arm_files` is the coverage arm's own predicate, run over the same
report in the same sweep, so the two can only disagree if someone edits that
predicate — which the new `sweep-dedup` outdatty group now gates.

The merged-arm answer is worse on inspection. The file and the function have
different remedies ("cover this file to the floor" vs "exercise these
branches"), different templates and different stem shapes, so one arm would
branch on granularity at every step and still file exactly the tasks the two
arms file now.

Measured on today's tree at CC >= 5 / coverage < 80: 10 findings, 4 of them
inside the 3 files the coverage arm reports, 6 in files it never will. At filing
the same band gave 16 and 8; the coverage arm's seven findings were closed
between then and now, which moved both totals and left the ratio — about half
unique — where it was. Verified live at both boundaries: with
`--coverage-floor 0` (the coverage arm silent) the arm reports all 10, with
`--coverage-floor 100` (every file deduped) it reports none.

=== The edges, and what each rejects

Coverage < 80, complexity >= 5, over 1025 production functions. Per-function
coverage has mean 97.61 and sd 7.97, so 80 is ~2.2 sd out — stricter than the
file arm's 88 at ~1.5 sd, because a function is small enough that one unreached
line moves its percentage by tens of points. The 80..89 band holds 63 functions
and reads as scatter (`confirm_retire` at 88.9% had just been fully tested);
below 80 sit 28 in the whole workspace. The complexity floor is not a complexity
judgment — `just crap` gates that at 30 and against a baseline — it is what makes
the coverage number mean something: the 18 functions at CC < 5 and coverage < 80
are `Display::fmt` impls, a test helper, a CC 1 function missing two of three
lines. Full numbers and the rejected neighbours are in the arm, next to the
constants.

The scoring runs off one derived quantity, `paths = CC * (1 - coverage)` — the
function's unexercised independent paths — so the priority and the finding talk
about the same thing. `hval_kind`, the case this task was filed for, comes out
top of the six at ice(7, 0.85, 4).
