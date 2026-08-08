#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "cover confirm_retire and unred the CRAP gate",
  priority: ice(
    impact: 9,
    confidence: 1.0,
    ease: 4,
  ),
  difficulty: 4,
  tags: ("tests", "cli"),
  estimate: (files: 2),
  links: related(
    "sweep-crap-scores-into-tasks.typ",
  )[measuring that arm's band is what surfaced this],
  status: proposed(2026, 8, 8),
)

== Summary

`just crap` is FAILING on `main`. `confirm_retire`
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

- Add an e2e test that drives `mt rm` down the retire path with a piped
  confirmation, the way the delete-path test already does. Retire mode needs a
  project whose gitless `rm` strategy is retire, so the fixture is the real
  cost here — check whether an existing fixture can be configured into it before
  adding another.
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
