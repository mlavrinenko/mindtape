#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "make the crap gate report only failures and refuse a stale lcov",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 8,
  ),
  difficulty: 2,
  tags: ("tooling",),
  estimate: (files: 3),
  status: done(
    2026,
    8,
    1,
  )[28 phantom failures to 1 real, report is 12 lines instead of 2400],
)

== Summary

`just crap` read `target/coverage/lcov.info` with no freshness check. Against a 10-day-old lcov it reported 28 failures, of which 27 were phantom. A function in a file the lcov never saw has no coverage record, and `--missing` defaults to `pessimistic`, so it scores as 0% covered rather than as unknown. Every phantom row showed coverage `—`, not a percentage. Regenerating coverage dropped 28 to 1. The report was also every analyzed function (1193 rows) with no way to see the failures. It scored the inline `*_tests.rs` modules as production code, 71 of the 73 `no matching entry in the LCOV report` warnings.

== Scope

- `--min` tracks `--threshold` so the table lists only failures. The tail summary then counts printed rows ("1/1", not "1/976"), which is cargo-crap behaviour and documented in the recipe
- exclude `**/*_tests.rs` and `**/tests.rs`. llvm-cov skips inline test modules and CRAP is a production-code risk metric. The verdict stays the same
- refuse to run when the lcov predates any `crates/**/*.rs`, naming `just cover`. mtime-based, and a bare `touch` trips it. It errs toward refusing rather than reporting invented failures
- CI unaffected: the `coverage` job already runs `just cover` immediately before
- out of scope: the one real finding the fresh run leaves, filed as cover-the-preset-command-entry-point
