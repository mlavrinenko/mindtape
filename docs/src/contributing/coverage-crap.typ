// docs/src/contributing/coverage-crap.typ renders
// docs/contributing/coverage-crap.md through docs/generate-md.sh (typlite;
// docs/src/lib.typ lists the constraints).
//
// The 90% line floor and the CRAP threshold of 30 are flags in
// .just/scripts/cover.sh and .just/scripts/crap.sh. No generated fact carries
// them, so this page is the one place that states them.
#import "../lib.typ": doc-title, just

#metadata((
  output: "docs/contributing/coverage-crap.md",
)) <mindtape-md>

#doc-title[Coverage and CRAP]

Why #just.cover() and #just.crap() work the way they do. Read this before you
change either recipe's flags.

= Coverage

#just.cover() runs the suite under `cargo-llvm-cov` in the last stage of
#just.check(), after every other gate passed. llvm-cov merges the profile of every
process, so the end-to-end tests that spawn `mt` count toward the command code
they run. The recipe writes `target/coverage/lcov.info` for #just.crap() and
fails below 90% of lines. `main.rs` is excluded, so keep it thin.

The output level lives in `.config/nextest.toml`, because
`cargo llvm-cov nextest` accepts `--status-level fail` and ignores it. With
the flag alone, a green run prints one `PASS` line per test, over 2000 lines.
#just.test() and #just.cover() both read the profile, so they print the same
way.

#just.cover() builds default features only, so `recur_rrule.rs` never
compiles there, and #just.crap() excludes it rather than score it as 0%
covered. #just.check-rrule() builds and tests the `rrule` feature across the
whole workspace, because `mindtape-cli` has `rrule` code of its own.

= Why cover starts clean

#just.cover() starts with `cargo llvm-cov clean --workspace`, so every run
pays a full instrumented rebuild of about a minute. Without the clean,
`report` merges every profile left since the last `cargo clean`, from other
feature sets and other commits, and reports coverage the current source
cannot produce. cargo-llvm-cov documents the clean as the precondition for
merging runs.

Inflated coverage hides exactly what #just.crap() and the sweep's coverage
arms exist to find. Measured here, a session's leftover profiles put a
function that no test can call at 84% covered. A baseline minted from such an
lcov fails dozens of functions on the next clean run. Never remove the clean
for speed.

= The CRAP threshold

#just.crap() runs after #just.cover() over the same `lcov.info`, in the same
CI job. CRAP is `CC² × uncovered³ + CC`, and the gate fails a function that
scores above 30. It refuses an lcov older than any source file, so run
#just.cover() first. It skips test modules (`*_tests.rs`, `tests.rs` and
ejectest's `*_tests_*.rs`), because llvm-cov emits no coverage for them and
they would score as undertested production code. `.just/scripts/crap.sh`
lists every exclude and why each is spelled `**/…`.

Above about 90% coverage the cubed term vanishes, and a function scores its
own cyclomatic complexity. A flat dispatcher sits near the threshold for that
reason, so adding an arm to one means covering it in the same change.

So the threshold is not the whole gate. #just.crap() also compares every
function against `crap-baseline.json`, a committed score per function, and
fails any function whose score rose. A rise means complexity went up or
coverage went down, at any score. That catches a complex, undertested function
that scores well under 30.

When you add complexity on purpose, run #just.crap-baseline() in the same
commit, so the new score lands beside the reason for it.
#link("docs/contributing/crap-baseline.md")[docs/contributing/crap-baseline.md]
covers the baseline file.

= Why a stale lcov is refused

A function in a file written after the lcov has no coverage record. With
`--missing` at its default, `pessimistic`, it scores as 0% covered. On a
10-day-old lcov that gave 28 failures, 27 of them false. A gate that reports
false failures is worse than none, so #just.crap() refuses a stale lcov and
names the recipe that refreshes it.

= Why the sweep has no CRAP arm

A CRAP band below 30 would point the wrong way. Of the 15 functions here that
score from 15 to 30, 11 are at least 90% covered. Of the 16 that are complex
and undertested, 12 score below 15. So #just.ask-refactoring() has no CRAP
arm.

= Why both run in every check

#just.check() runs #just.cover() and then #just.crap() as its last stage, so
the pre-commit hook refuses a commit whose coverage or CRAP score was never
measured. A CRAP gate that skipped on a stale lcov would skip on nearly every
commit that changes code, and the baseline would drift unseen. The stage costs an
instrumented clean rebuild, about two minutes, and runs only after the faster
stages pass.

= Where the tools come from

`flake.nix` supplies cargo-crap and the LLVM tools `cargo-llvm-cov` needs, and
its comments explain both.
