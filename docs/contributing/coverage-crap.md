<!-- Generated from docs/src/contributing/coverage-crap.typ by `just docs md`. Do not edit; edit the source. -->

# Coverage and CRAP

Why `just cover` and `just crap` work the way they do. Read this before you change either recipe’s flags.

## Coverage

`just cover` runs the suite under `cargo-llvm-cov` in the last stage of `just check`, after every other gate passed. llvm-cov merges the profile of every process, so the end-to-end tests that spawn `mt` count toward the command code they run. The recipe writes `target/coverage/lcov.info` for `just crap` and fails below 90% of lines. `main.rs` is excluded, so keep it thin.

The workspace total can stay green while one file sinks, so the recipe also gates each production file. It fails a file below 88% line coverage that has 5 or more uncovered lines, and names it with its percentage and its unreached line count. Test files don’t count as production. The recipe reads each record’s `LF` and `LH` from the lcov, not `cargo llvm-cov report --json`, whose summary counts a line once per compiled copy of the `mt` crate and shows a fully covered file at 75%.

88% sits about 1.5 standard deviations under the per-file mean (95.97%, sd 5.32 over 125 files when measured). That flags a real tail, not the scatter of files a point or two under the average. The 5-line minimum keeps a small file out of the gate when 3 lines of tty or stdio wiring sink its percentage. The fix is a test. If the lines can’t be tested, add the file to `ignore` in `.just/scripts/cover.sh` with the reason beside it.

The output level lives in `.config/nextest.toml`, because `cargo llvm-cov nextest` accepts `--status-level fail` and ignores it. With the flag alone, a green run prints one `PASS` line per test, over 2000 lines. `just test` and `just cover` both read the profile, so they print the same way.

## Why cover starts clean

`just cover` starts with `cargo llvm-cov clean --workspace`, so every run pays for a full instrumented rebuild of about a minute. Without the clean, `report` merges every profile left since the last `cargo clean`, from other feature sets and other commits. It then reports coverage the current source can’t produce. cargo-llvm-cov documents the clean as the precondition for merging runs.

Inflated coverage hides exactly what `just crap` and the sweep’s undertested arm exist to find. Measured here, a session’s leftover profiles put a function that no test can call at 84% covered. A baseline minted from such an lcov fails dozens of functions on the next clean run. Don’t remove the clean to save time.

## The CRAP threshold

`just crap` runs after `just cover` over the same `lcov.info`, in the same CI job. CRAP is `CC² × uncovered³ + CC`, and the gate fails a function that scores above 30. It refuses an lcov older than any source file, so run `just cover` first. It skips test modules (`*_tests.rs`, `tests.rs` and ejectest’s `*_tests_*.rs` and `tests_*.rs`). llvm-cov emits no coverage for them, so they’d score as undertested production code. `.just/scripts/crap.sh` lists every exclude and why each is spelled `**/…`.

Above about 90% coverage the cubed term vanishes, and a function scores its own cyclomatic complexity. That’s why a flat dispatcher sits near the threshold. If you add an arm to one, cover it in the same change.

So the threshold isn’t the whole gate. `just crap` also compares every function against `crap-baseline.json`, a committed score per function, and fails any function whose score rose. A rise means complexity went up or coverage went down, at any score. That catches a complex, undertested function that scores well under 30. It also fails a function the baseline doesn’t list, and names it. Otherwise a new function stays unheld until someone re-mints, and that mint buries one real rise among every function added since.

When you add a function or add complexity on purpose, run `just crap-baseline` in the same commit, so the new score lands beside the reason for it. [docs/contributing/crap-baseline.md](docs/contributing/crap-baseline.md) covers the baseline file.

## Why a stale lcov is refused

A function in a file written after the lcov has no coverage record. With `--missing` at its default, `pessimistic`, it scores as 0% covered. On a 10-day-old lcov that gave 28 failures, 27 of them false. A gate that reports false failures is worse than none, so `just crap` refuses a stale lcov and names the recipe that refreshes it.

## Why the sweep has no CRAP arm

A CRAP band below 30 would point the wrong way. Of the 15 functions here that score from 15 to 30, 11 are at least 90% covered. Of the 16 that are complex and undertested, 12 score below 15. So `just ask-refactoring` has no CRAP arm.

## Why both run in every check

`just check` runs `just cover` and then `just crap` as its last stage, so the pre-commit hook refuses a commit whose coverage or CRAP score was never measured. A CRAP gate that skipped on a stale lcov would skip on nearly every commit that changes code, and the baseline would drift unseen. The stage costs an instrumented clean rebuild, about two minutes, and runs only after the faster stages pass.

## Where the tools come from

`flake.nix` supplies cargo-crap and the LLVM tools `cargo-llvm-cov` needs, and its comments explain both.
