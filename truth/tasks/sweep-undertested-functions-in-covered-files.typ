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
  status: proposed(2026, 8, 8),
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
