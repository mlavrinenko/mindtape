# ask-refactoring: the coverage arm — per-file line coverage at --coverage-floor.
#
# Sourced by `contrib/ask-refactoring.sh`; not executable on its own. It reads
# that script's globals (`$repo`, `$MT`, `$WRITE`, the thresholds) and calls its
# `stem_for`. It owns three functions the driver looks up by name —
# `score_coverage`, `signal_coverage`, `emit_coverage` — reading the lcov `just cover` writes.
#
# The reasoning behind this arm's numbers lives here with the code that
# applies them, deliberately: it is the most valuable thing in the arm and a
# doc it lived in instead would drift away from the constants it explains.

# ── signal: coverage ────────────────────────────────────────────────────────
#
# `just cover` gates the WORKSPACE TOTAL at 90% lines (`--fail-under-lines 90`
# in the Justfile) — it does not gate any individual file. So a file sitting
# below the band this arm reports is not "drifting toward the gate": nothing
# gates it at all. That is exactly why it is advisory material for this sweep
# and not a duplicate of the `just cover` gate — the two would only overlap if
# per-file coverage were itself enforced, and it is not.
#
# The lcov `just cover` writes is a build artifact nothing re-derives for free.
# `just crap`'s own comment (see the Justfile) documents at length what a stale
# one does: a source the lcov never saw has no coverage record at all, so it
# reads as 0% covered rather than as unknown — a phantom failure, not a missing
# measurement. `ensure_lcov_fresh` below runs the exact same two checks `just
# crap` runs (missing file, then a source newer than it) after asking mmz to
# refresh the lcov, so a stale one is refused here for the same reason and
# named the same way, whether or not mmz's own cache is what refreshed it.
#
# `just mmz cover` (a new rule in .mmz/config.yaml, scoped like `just test`
# since cargo-llvm-cov runs the same suite) is the memoization the task calls
# for: a re-sweep of a tree nothing has touched since the last `just cover`
# skips the whole instrumented test run and prints mmz's on_hit note instead.
# This is the one place in this script that reaches for mmz, and deliberately
# so — see the file header. Re-running the actual test suite on every sweep
# would make the coverage arm the slowest thing about `just ask-refactoring`
# for no benefit: the measurement does not change unless the sources do.
ensure_lcov_fresh() {
    # Onto stderr, never stdout. This runs inside `signal_coverage`, whose whole
    # stdout IS the TSV row stream, so on an mmz cache MISS the instrumented test
    # run prints straight into the rows — "running 939 tests" then parses as a
    # finding with an empty score and fails the fill. A cache hit hides it, which
    # is why it survives casual testing: the arm only misbehaves on the runs
    # where it is doing the work it exists to do. Progress still reaches the
    # terminal, because a silent multi-minute pause reads like a hang.
    just mmz cover >&2
    local lcov="target/coverage/lcov.info"
    if [ ! -f "$lcov" ]; then
        echo "ask-refactoring: $lcov is missing — run \`just cover\` first" >&2
        exit 1
    fi
    local newer
    newer=$(find crates -name '*.rs' -newer "$lcov" -print -quit)
    if [ -n "$newer" ]; then
        echo "ask-refactoring: $lcov predates $newer — run \`just cover\` first" >&2
        echo "  (a source the lcov never saw scores as 0% covered, not as unknown)" >&2
        exit 1
    fi
}

# Band edge, and why 88 and not a round 90 (the workspace-total gate's number,
# which would misleadingly imply this is the same band):
#
#   On today's tree (94.84% total, 125 production, non-test, non-stub files),
#   per-file line coverage has mean 95.97%, standard deviation 5.32. 88% is
#   ~1.5 standard deviations below the mean — far enough out that it is
#   reporting a real tail, not the ordinary scatter of files sitting a point or
#   two under the workspace average. It reports 7 files today: rm_gitless_report.rs
#   (57.30%, the one real outlier), review.rs (84.31%), framework_eval.rs
#   (85.63%), emit.rs (86.55%), config_cmd.rs (87.10%), condition.rs (87.15%),
#   import.rs (87.92%). A handful, same as `--baseline 90` reports a handful of
#   oversized files above — not the forty-plus files sitting in the 88-95% band
#   that would make this arm noise instead of signal.
#
# Excluded before the band is even applied:
#
#   Test files. `*_tests.rs` / `tests.rs` (and ejectest's infix-split
#   `foo_tests_bar.rs` siblings) are instrumented and would otherwise be swept
#   as if they were undertested production code, when a low percentage there
#   usually just means an early-return branch in a test helper nothing hits.
#   `just crap` already excludes the same files from CRAP scoring (`--exclude
#   '**/*_tests.rs' --exclude '**/tests.rs'` in the Justfile) and its comment
#   there explains why (inline `#[path] mod tests` modules that llvm-cov
#   attributes no usable records to). The regex below is jq's equivalent of
#   those two globs — verified to select the identical 110-file set on this
#   tree — written as one pattern because jq has no glob matcher.
#
#   recur_rrule_off.rs. The `#[cfg(not(feature = "rrule"))]` stub that compiles
#   under the default feature set `just cover` builds with. `.cargo/mutants.toml`
#   already excludes it from mutation testing for the same reason: it is a
#   handful of thin forwarding functions, not undertested logic, and scores low
#   only because there is almost nothing there to cover. (`recur_rrule.rs`, the
#   real implementation behind the `rrule` feature, does not appear in the
#   report at all under the default build — nothing compiles it, so llvm-cov
#   has no records for it either way; `just crap` excludes it explicitly because
#   its own tool walks the source tree independently of what compiled.)
#
# Both constants are read by the undertested arm too (see its header): that arm
# skips a function whose FILE this one reports, and the dedup is only sound if the
# two agree on which files are production code to begin with. One definition, so
# they cannot drift apart.
COVERAGE_TEST_RE='(^|/)([^/]*_)?tests(_[^/]*)?\.rs$'
COVERAGE_STUB='crates/mindtape/src/recur_rrule_off.rs'

score_coverage() {
    jq -r --arg repo "$repo/" --argjson floor "$COVERAGE_FLOOR" \
        --arg re "$COVERAGE_TEST_RE" --arg stub "$COVERAGE_STUB" '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      .data[0].files[]
      | select(.summary.lines.count > 0)
      | (.filename | ltrimstr($repo))                       as $path
      | select(($path | test($re)) | not)
      | select($path != $stub)
      | .summary.lines                                      as $l
      | $l.percent                                          as $percent
      | $l.count                                            as $lines
      | ($lines - $l.covered)                                as $uncovered
      | select($percent < $floor)
      | (100 - $percent)                                    as $dark
      | (($dark / 5 | round) | clamp(1; 10))                as $impact
      | ((9 - ($uncovered / 5 | round)) | clamp(1; 10))     as $ease
      | (if $uncovered <= 15 then 2
         elif $uncovered <= 30 then 4 else 6 end)            as $difficulty
      | [
          $path,
          (($percent * 100 | round) / 100 | tostring),
          ($uncovered | tostring),
          ($lines | tostring),
          ($floor | tostring),
          ($impact | tostring),
          "0.9",
          ($ease | tostring),
          ($difficulty | tostring)
        ]
      | @tsv
    '
}

# impact = round(dark% / 5), clamped 1..10, where dark% = 100 - percent
#   How much of the file the suite never reaches. A file 40+ points under the
#   floor (the rm_gitless_report.rs case) is a materially different problem
#   from one a couple of points under it, so impact tracks the raw gap rather
#   than the file's rank in a short list.
#
# confidence = 0.9, constant
#   High because llvm-cov COUNTS: every line's coverage is instrumentation
#   data, not a guess, matching the template's own prompt. Not 1.0 for the same
#   reason `score_size` stops at 0.9 for production code — the remedy (test the
#   gap, or exclude the file with a reason) is still a judgment call the number
#   alone does not make. There is no lower test-file tier the way `score_size`
#   has one, because test files are excluded outright above rather than scored.
#
# ease = clamp(9 - round(uncovered / 5), 1, 10)
#   `uncovered` is the count of never-reached lines — the literal amount of new
#   test material required. Ease falls as that count rises, same shape as
#   `score_size`'s `move`-based formula.
#
# difficulty: 2 for <=15 uncovered lines, 4 for <=30, else 6, on the 0-9
#   judgment scale the template asks for. Matches `score_size`'s three-tier
#   `move` rubric: a handful of lines is one obvious test, dozens start to
#   need real fixture setup.
#
# estimate is a flat "(files: 1)" for every finding, not derived per-row.
#   Closing a coverage gap adds cases to a test module that (on this tree)
#   already exists for every file this arm reports — it does not invent a new
#   production module the way a size split does, so there is no second file to
#   count. Markdown's flat 2 in `score_size` is the same move for the opposite
#   reason: a constant is honest when the real answer does not vary with the
#   measurement, not when it is merely convenient.
# The producer, split out of `signal_coverage` because `coverage_arm_files` needs
# the same report. Captured with `$(...)` and rc-checked rather than piped blind:
# llvm-cov failing quietly would read as "no findings", and the sweep's one
# unaffordable failure is a clean, wrong result. The `exit 1` reaches the top
# level through `set -eo pipefail` — the pipeline element it kills is a subshell,
# but pipefail carries its status out.
coverage_report_json() {
    local ignore='crates/mindtape-cli/src/main\.rs|/registry/src/'
    local json rc
    set +e
    json="$(cargo llvm-cov report --ignore-filename-regex "$ignore" --json --summary-only)"
    rc=$?
    set -e
    if [ $rc -ne 0 ] || ! printf '%s' "$json" | jq -e '.data[0].files' >/dev/null 2>&1; then
        echo "ask-refactoring: cargo llvm-cov report failed (exit $rc) or wrote no JSON" >&2
        exit 1
    fi
    printf '%s' "$json"
}

signal_coverage() {
    ensure_lcov_fresh
    coverage_report_json | score_coverage
}

# The paths this arm reports, and the whole of the undertested arm's dedup rule:
# a function whose file appears here is inside this arm's finding already, so the
# function-shaped arm stays quiet about it. It is this arm's own predicate rather
# than a copy of it — same report, same floor, same run — so the two arms can
# only disagree about a file if someone edits `score_coverage`.
coverage_arm_files() {
    coverage_report_json | score_coverage | cut -f1
}

emit_coverage() {
    local path="$1" percent="$2" uncovered="$3" lines="$4" floor="$5"
    local impact="$6" confidence="$7" ease="$8" difficulty="$9"

    local args=(
        add "cover $path"
        -t sweep-coverage
        --name "$(stem_for coverage "$path")"
        --set "path=$path"
        --set "percent=$percent"
        --set "uncovered=$uncovered"
        --set "lines=$lines"
        --set "floor=$floor"
        --set "impact=$impact"
        --set "confidence=$confidence"
        --set "ease=$ease"
        --set "difficulty=$difficulty"
        --set "estimate=(files: 1)"
        --set 'tags[]=swept' --set 'tags[]=coverage'
    )

    if [ "$WRITE" -eq 1 ]; then
        local out outcome dest
        out="$("$MT" "${args[@]}" --dedup skip --format json)"
        outcome="$(printf '%s' "$out" | jq -r '.outcome')"
        dest="$(printf '%s' "$out" | jq -r '.path')"
        case "$outcome" in
            wrote) echo "filed   $dest" ;;
            skipped)
                echo "skipped $dest" >&2
                echo "        the stem is taken, so this finding was filed before and closed" >&2
                echo "        without the file's coverage rising — reopen it, or exclude it in" >&2
                echo "        \`just cover\` with a written reason" >&2
                ;;
            *) echo "$outcome $dest" ;;
        esac
    else
        echo "── would file: $(stem_for coverage "$path")  [ice($impact, $confidence, $ease)]"
        "$MT" "${args[@]}" --print
        echo
    fi
}
