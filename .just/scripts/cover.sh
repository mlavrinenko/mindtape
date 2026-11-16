#!/usr/bin/env bash
# .just/scripts/cover.sh: the body of the `cover` recipe, split out under
# `truth/tasks/extract-the-coverage-and-release-recipe-bodies-into-scripts.typ`.
# Runs from the repo root, like the recipe that invokes it.
# docs/contributing/coverage-crap.md argues why cargo-llvm-cov replaced
# tarpaulin and why the `clean --workspace` below is the line that makes the
# number mean anything.
set -eo pipefail
# The regex excludes main.rs to keep it thin (`just crap::check` excludes it
# too), plus registry checkouts, which llvm-cov's dep filter misses when
# CARGO_HOME relocates (a local kache). So local and CI report the same set.
ignore='crates/mindtape-cli/src/main\.rs|/registry/src/'
# Test files are not production code: the same regex as COVERAGE_TEST_RE in
# contrib/ask-refactoring/undertested.sh, jq's equivalent of `just crap::check`'s globs.
test_re='(^|/)([^/]*_)?tests(_[^/]*)?\.rs$'
mkdir -p target/coverage
# Instrument once (`--no-report`), then render both reports from that profdata.
# The clean is cargo-llvm-cov's own documented precondition for that split
# ("should be called before the first run"). `--no-report` deliberately does
# not clear the profraw pile: the split exists to merge more than one run.
# Dropping it unions every profraw and every object file left in
# target/llvm-cov-target since the last `cargo clean` into the lcov. Measured,
# and argued in the docs page above.
cargo llvm-cov clean --workspace
# `nextest` subcommand, matching `just test`'s own move to nextest. Gating a
# suite under one execution model while measuring it under another can pass in
# one and fail in the other with nothing saying so. llvm-cov merges the
# profraw of child processes, so subprocess-based CLI e2e tests still
# attribute to the code they exercise. `--cargo-quiet` twice drops cargo's
# build lines, as in `just test`.
cargo llvm-cov nextest --workspace --no-report --cargo-quiet --cargo-quiet
cargo llvm-cov report --ignore-filename-regex "$ignore" \
    --lcov --output-path target/coverage/lcov.info
# Only the header and `TOTAL` row: the per-file table is ~230 lines a gate
# run never reads, `just crap::check` reads the lcov above, and `--summary-only`
# drops nothing in text mode. pipefail keeps `--fail-under-lines`' exit status.
cargo llvm-cov report --ignore-filename-regex "$ignore" --fail-under-lines 90 |
    sed -n '1p;$p'

# The per-file floor fails a production file below 88% line coverage when it
# also has 5 or more uncovered lines. Read from every lcov record's line
# totals, not from `cargo llvm-cov report --json`. Its summary counts a line
# once per compiled `mt` copy, so a file the suite covers whole reads as 75%.
#
# Why 88, and not the workspace total's 90: the workspace total can stay green
# while one file sinks. 88 is ~1.5 standard deviations under the per-file mean
# (95.97%, sd 5.32 over 125 files when measured). That reports a real tail
# rather than the scatter of files a point or two under the average. The 5-line
# minimum exists because in a small file 3 lines of tty or stdio wiring sink it
# below the floor. A test for them is not worth writing.
#
# The fix is a test. If you cannot test the lines, add the file to `ignore`
# above with the reason beside it.
floor=88
min_uncovered=5
# The patterns go in through the environment: awk -v would eat their backslashes.
below="$(ignore="$ignore" test_re="$test_re" awk -v root="$PWD/" \
    -v floor="$floor" -v min="$min_uncovered" '
    BEGIN { ignore = ENVIRON["ignore"]; test_re = ENVIRON["test_re"] }
    /^SF:/ { f = substr($0, 4); sub("^" root, "", f) }
    /^LF:/ { lf = substr($0, 4) + 0 }
    /^LH:/ { lh = substr($0, 4) + 0 }
    /^end_of_record/ {
        if (f !~ ignore && f !~ test_re && lf > 0 && lf - lh >= min \
            && lh * 100 / lf < floor)
            printf "  %s  %.1f%%  %d of %d lines unreached\n", \
                f, lh * 100 / lf, lf - lh, lf
        f = ""; lf = 0; lh = 0
    }
' target/coverage/lcov.info | sort)"
if [ -n "$below" ]; then
    echo "cover: $(printf '%s\n' "$below" | wc -l | tr -d ' ') file(s) below the per-file floor of ${floor}% (${min_uncovered}+ uncovered lines):"
    printf '%s\n' "$below"
    echo "fix: add a test, or add the file to \`ignore\` in .just/scripts/cover.sh with the reason beside it"
    exit 1
fi

# The changed-line floor catches a branch that sits above every absolute floor
# while its new guards have no tests. `git diff` rather than `--compare-branch`
# so staged and working-tree lines count too. A plain `git diff <ref>` includes
# both. `main` itself has an empty diff and passes. A checkout without the ref
# (a shallow CI clone) says so rather than failing on git.
base="${MINDTAPE_COVER_BASE:-${GITHUB_BASE_REF:-main}}"
if git rev-parse --verify --quiet "$base" >/dev/null; then
    diff_file=target/coverage/changed.diff
    git diff "$base" -- > "$diff_file"
    if [ -s "$diff_file" ] &&
        ! diff-cover target/coverage/lcov.info --diff-file "$diff_file" \
            --exclude '*_tests*.rs' 'tests*.rs' --fail-under=100; then
        echo "cover: new or changed lines must be covered; add a test for the lines named above"
        exit 1
    fi
else
    echo "cover: no '$base' ref; skipping the changed-line floor (set MINDTAPE_COVER_BASE)" >&2
fi
