#import "@local/mindtape:0.3.0": *

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
  )[built: `dedup` by deferral to the file-shaped arm, band at CC 5 / coverage 80],
)

== Summary

`sweep-crap-scores-into-tasks` measured its way to cancellation: CRAP degenerates
to cyclomatic complexity above ~90% coverage, so a band below the CRAP-30 gate
reports well-tested complex functions and misses undertested ones. The reasoning
and the numbers are in that task.

The measurement did establish that the shape was right even though the metric was
wrong. The coverage arm is file-shaped, and a file passes the 88% floor while
still hiding a function nobody tests. `hval_kind`
(`crates/mindtape/src/model.rs`) sits at 40% coverage inside a file the coverage
arm never reports. A file-shaped signal is structurally unable to see that.

The candidate signal is the conjunction the CRAP metric only approximates. It
reads cyclomatic complexity at or above some floor and line coverage below some
ceiling from the same lcov, per function. At CC >= 5 / coverage < 80% it finds 16
functions today, and 8 of them are in files the coverage arm never reports. Those
8 are the whole argument for this arm. The other 8 already have a file-shaped
task and would be duplicates.

Confidence is 0.7 because that overlap is the open question, not the thresholds.

== Scope

- Decide the deduplication first, before any thresholds. Half of today's findings
  are in files the coverage arm already files a task for. Two arms filing about
  the same file is how the re-run guard ends up blocking on twice the work.
  Either this arm skips functions whose file the coverage arm reports, or the two
  become one arm that chooses its unit per finding. That choice settles whether
  this is worth building at all.
- Pick CC and coverage edges from the real distribution the way the coverage arm
  picked 88, and record the numbers with the reasoning next to the code. Since
  `bring-the-sweep-script-under-a-size-cap`, that code lives in
  `contrib/ask-refactoring/<arm>.sh`, with the arm named in `ARMS`.
- Read the same lcov, and honour `ensure_lcov_fresh` rather than adding a second
  staleness mechanism. Complexity can come from `cargo crap --format json`,
  which already reports `cyclomatic` per function alongside `coverage`. Using it
  as a complexity source is not the same as banding on its CRAP score, and that
  distinction is what ruled out the previous design.
- The remedy is the coverage arm's, not the size arm's: write the missing test,
  or exclude with a written reason. The remedy for a function at CC 5 and 57%
  coverage is not a split.
- One task per function, so the stem must carry the function name. `stem_for`
  namespaces by signal already. The mutants arm's stem rule is the precedent:
  derive from what identifies the finding, never a line number, so the stem
  survives edits above it.
- Cancelling this is still a real outcome. Data cancelled its predecessor, and
  the same standard applies. If the deduplication question does not have a clean
  answer, this arm files noise beside an arm that already works.

== Outcome: built, and the deduplication answer that decided it

The arm lands as `contrib/ask-refactoring/undertested.sh` with
`.mindtape/templates/sweep-undertested.typ`, named in `ARMS` after `coverage`.

=== The deduplication, answered first

The two candidate answers were "skip functions whose file the coverage arm
reports" and "one arm that picks its unit per finding". The first is right, and
not as a compromise: the remedies nest. A file below the coverage floor gets one
task whose work is covering the functions inside it. A function-level task beside
it is that task's first bullet with a stem of its own: pure duplication.
Under the re-run guard a duplicate pair blocks the next sweep until both halves
close. Above the floor, this arm is the only reporter and still reports every
one. The skip is exact rather than heuristic: `coverage_arm_files` is the
coverage arm's own predicate, run over the same report in the same sweep. The two
can disagree only if someone edits that predicate, and the new `sweep-dedup`
outdatty group gates it.

The merged-arm answer is worse on inspection. The file and the function have
different remedies ("cover this file to the floor" vs "exercise these branches"),
different templates and different stem shapes. One arm would branch on unit at
every step and still file exactly the tasks the two arms file today.

Measured on today's tree at CC >= 5 / coverage < 80: 10 findings, 4 of them
inside the 3 files the coverage arm reports, 6 in files it never will. At filing
the same band gave 16 and 8. The coverage arm's seven findings closed between
that filing and this measurement, which moved both totals and left the ratio,
about half unique, where it was. Verified live at both boundaries: with `--coverage-floor 0` (the
coverage arm silent) the arm reports all 10, and with `--coverage-floor 100`
(every file deduped) it reports none.

=== The edges, and what each rejects

Coverage < 80, complexity >= 5, over 1025 production functions. Per-function
coverage has mean 97.61 and sd 7.97, so 80 is ~2.2 sd out, stricter than the file
arm's 88 at ~1.5 sd. A function is small enough that one unreached line moves its
percentage by tens of points. The 80..89 band holds 63 functions and reads as
scatter (`confirm_retire` at 88.9% had just been fully tested), and below 80 sit
28 in the whole workspace. The complexity floor is not a complexity judgment.
`just crap` gates that at 30 and against a baseline. The floor is what makes the
coverage number mean something. The 18 functions at CC < 5 and coverage < 80 are
`Display::fmt` implementations, a test helper, a CC 1 function missing two of
three lines. Full numbers and the rejected neighbours are in the arm, next to the
constants.

The scoring runs off one derived quantity, `paths = CC * (1 - coverage)`, the
function's unexercised independent paths, so the priority and the finding talk
about the same thing. `hval_kind`, the case that prompted this task, comes out
top of the six at ice(7, 0.85, 4).
