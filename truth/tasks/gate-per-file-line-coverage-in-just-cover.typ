#import "@local/mindtape:0.2.0": *
#import "../lib.typ": *

#show: task.with(
  title: "Gate per-file line coverage in just cover",
  kind: "dev",
  priority: ice(
    impact: 7,
    confidence: 0.85,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling", "gating"),
  estimate: (src-files: 8),
  x-goals: (
    product: (
      readable-tasks: neutral,
      implementable-tasks: neutral,
      enforced-practice: helps[a file can no longer sink below the floor while the workspace total stays green],
      cheap-adoption: neutral,
      typst-over-markdown: neutral,
      plain-files: neutral,
    ),
    process: (
      sound-codebase: helps[coverage debt cannot pile up between sweeps],
      docs-match-code: neutral,
    ),
  ),
  assignee: "agent",
  status: proposed(2026, 10, 13),
)

== Problem

+ Run `just cover` on a tree where one production file sits at 50% line coverage and the total stays above 90%.
+ 🔴 It passes: `--fail-under-lines 90` gates the workspace total only, so per-file coverage is held by nothing but the sweep's coverage arm, which files tasks after the fact.

== TO-BE

- 🟢 `just cover` fails naming every production file below 88% line coverage with 5 or more uncovered lines, read from `target/coverage/lcov.info`.
- 🟢 The sweep has no coverage arm and no `--coverage-floor`; the undertested arm keeps the lcov helpers it borrowed.

```
cover: 1 file(s) below the per-file floor of 88% (5+ uncovered lines):
  crates/mindtape-cli/src/cmd/review.rs  79.1%  14 of 67 lines unreached
```

== Changes

- `.just/scripts/cover.sh`: the per-file check after the lcov is written.
- `contrib/ask-refactoring/coverage.sh`: removed; `ensure_lcov_fresh` and `COVERAGE_TEST_RE` move to `undertested.sh`.
- `contrib/ask-refactoring.sh`, `Justfile`, `.just/scripts/arg-pattern-check.sh`, `www/utils/sweep-flag-notes.typ`: drop the arm and its flag.
- `docs/src/contributing/coverage-crap.typ`, `docs/src/contributing/signal-sweep.typ`: the per-file gate, the arm gone.
- `.mindtape/templates/sweep-coverage.typ`, `outdatty.yaml`: the template and its groups go.

== Rollback

- Revert the commit; the arm and its flag come back with it.

== Out of scope

- The workspace-total number and the CRAP gate.
- A clone gate at the dry arm's bar, which waits on `merge-the-jscpd-baseline-by-fingerprint-union`.

== Dogfood

This repo's `just check` refuses a commit that leaves a file under 88%.
