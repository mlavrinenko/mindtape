#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "cover confirm_retire and unred the CRAP gate",
  priority: ice(
    impact: 9,
    confidence: 1.0,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("tests", "cli"),
  estimate: (files: 2),
  links: related(
    "sweep-crap-scores-into-tasks.typ",
  )[measuring that arm's band is what surfaced this],
  status: done(
    2026,
    8,
    8,
  )[e2e drives both prompt halves; the gate exits 0 on a fresh lcov],
)

== Summary

`just crap` was FAILING on `main` when this was filed. `confirm_retire`
(`crates/mindtape-cli/src/rm_gitless_report.rs:76`) scores CRAP 72.0 — cyclomatic
8, coverage 0.0% — against the gate's threshold of 30. The gate is CI-enforced
and is not part of `just check`, which is why a green local `just check` has been
hiding it.

It is not a stale-lcov phantom, and that was checked rather than assumed. The
lcov carries a real record for the function — `FNDA:0` in both binaries — while
every sibling in the same file is executed: `confirm_answer` 15 hits,
`report_retarget` 25, `report_retire_applied` 20, `report_retire_plan` 5. So the
suite reaches this file thoroughly and steps around exactly one function.

The reason is the shape of the tests, not an oversight in the file.
`confirm_answer` reads stdin and is exercised by the subprocess e2e tests
(`cargo-llvm-cov` sees across them), but only through the DELETE path —
`rm.rs:445` calls it directly. The retire path at `rm.rs:216` goes through
`confirm_retire`, and nothing drives `mt rm` in retire mode interactively.

== Scope

- Add an e2e test that drives `mt rm` down the retire path without `-y`, so the
  prompt is reached. The fixture is not the cost: `rm_e2e.rs` already holds
  `RETIRE_CONFIG`, `RETIRE_STATUS_CONFIG` and `RETIRE_ARCHIVE_CONFIG` (lines
  23--27), and every existing retire test passes `-y` to skip the prompt. The
  delete path's 15 hits on `confirm_answer` come from tests that omit `-y` and
  let the empty stdin read as "no"; the same trick reaches `confirm_retire`, and
  `assert_cmd`'s `write_stdin("y\n")` reaches the accepting branch.
- The function is CC 8 and writes four conditional blocks (the rename line, the
  link-retarget line) before delegating to `confirm_answer`. One pass through it
  will not cover it; the plan variants are the test cases.
- Do not reach for `--allow` or an `--exclude` to silence this. The exclusions in
  `just crap` are for code the lcov cannot see (`recur_rrule.rs` behind a feature,
  inline test modules); this is production code the suite genuinely never runs.
- Done when `just crap` exits 0 after a fresh `just cover`. Re-run `just cover`
  first — the recipe refuses an lcov older than the sources, and the fix changes
  them.
- Worth deciding separately, and not in this task: `just crap` sits outside
  `just check`, so only CI notices it. That is how it went red unobserved.

== Outcome

Two tests in `crates/mindtape-cli/tests/rm_e2e.rs`, one per half of the
function. Omitting `-y` reaches the prompt at all — the empty stdin reads as
"no", covering the plan listing and the abort; the archive fixture plus a piped
`y` covers the `rename` and `link_rewrites` branches and the accepting answer.

One thing the plan had backwards: retiring IN PLACE produces no link rewrites,
because the file keeps its path and no inbound edge moves. The retarget line
prints only under `rename`. Asserted as an absence in the in-place test, since
that absence is what separates the two cases.

`confirm_retire` went 0% → 88.9% (CRAP 72.0 → 8.1) and the gate exits 0. The
file went 51/89 lines → 75/89, which is 84% and still under the sweep's 88
floor: covering the gate's function did not clear the file-shaped finding, and
the 14 lines left are a separate, smaller piece of work. The separate decision
named above is filed as `observe-the-crap-gate-locally.typ`.
