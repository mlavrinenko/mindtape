# ask-refactoring: the undertested arm — per-FUNCTION complexity x coverage.
#
# Sourced by `contrib/ask-refactoring.sh`; not executable on its own. It reads
# that script's globals (`$repo`, `$MT`, `$WRITE`, the thresholds) and calls its
# `stem_for`. It owns three functions the driver looks up by name —
# `score_undertested`, `signal_undertested`, `emit_undertested` — reading
# `cargo crap --format json` over the lcov `just cover` writes.
#
# It also reads three things from `coverage.sh` rather than reimplementing them:
# `ensure_lcov_fresh`, `coverage_arm_files`, and the `COVERAGE_TEST_RE` /
# `COVERAGE_STUB` production-file constants. The dedup below is sound only if the
# two arms agree on what production code is and on which files the file-shaped arm
# already reports, and agreement between two copies of a predicate decays. `ARMS`
# lists this arm after `coverage` for that reason; the `sweep-dedup` outdatty group
# is the gate on it.
#
# The reasoning behind this arm's numbers lives here with the code that applies
# them, deliberately: a doc it lived in instead would drift from the constants.

# ── signal: complexity x coverage, per function ─────────────────────────────
#
# A file-shaped signal is structurally unable to see an undertested function
# inside a well-covered file. `hval_kind` (`crates/mindtape/src/model.rs`) is 40%
# covered at cyclomatic complexity 8, in a file the coverage arm never reports,
# under a `just cover` gate that measures the workspace total only. Nothing else
# in the repo reports it; that is this arm's entire reason to exist.
#
# The band is the conjunction — complexity at or above a floor and coverage below
# one — never the CRAP score that combines them. CRAP is CC^2*(1-cov)^3 + CC,
# whose cubed term collapses above ~90% coverage, so in this repo it degenerates
# into cyclomatic complexity and no band on it separates a complex well-tested
# function from an undertested one; `truth/tasks/sweep-crap-scores-into-tasks.typ`
# measured that and was cancelled on it. `cargo crap --format json` is still the
# right source — nothing else here reports `cyclomatic` and `coverage` per
# function — and reading those two fields is not banding on the score printed
# beside them. That is the difference between this arm and the cancelled one.
#
# Freshness is `ensure_lcov_fresh`, called again rather than duplicated: a source
# the lcov never saw has no record, so `cargo crap` reads it as 0% covered rather
# than as unknown, and a sweep that files phantoms is worse than one that refuses.
# Arms run in separate command substitutions, so this second call cannot see that
# the first happened; it costs an `mmz` cache hit, not a second test run.
#
# Exclusions come from `just _crap-excludes` — the Justfile's `CRAP_EXCLUDES`,
# whose comment there gives the reason a second hand-maintained copy is not
# acceptable; a third in a shell script is the same mistake. Those globs drop
# `main.rs` (absent from the lcov, so every function in it would score 0%),
# `recur_rrule.rs` (feature-gated, uncompiled under the default build) and the
# `*_tests.rs` / `tests.rs` modules. On top of them this arm applies the coverage
# arm's two production filters, which catch what the globs miss: ejectest's
# infix-split siblings (`add_tests_subprocess.rs`) and the `recur_rrule_off.rs`
# stub. A file the coverage arm calls non-production has to be non-production
# here too, or the dedup would pass a test helper through on the grounds that the
# other arm never reports it.
undertested_report_json() {
    local excludes
    # `read -r -a` rather than an unquoted expansion: the list is globs, and a
    # bare word-split would let the shell expand `**/main.rs` against the tree.
    read -r -a excludes <<<"$(just _crap-excludes)"
    local json rc
    set +e
    json="$(cargo crap --lcov target/coverage/lcov.info --workspace \
        "${excludes[@]}" --min 0 --format json)"
    rc=$?
    set -e
    if [ $rc -ne 0 ] || ! printf '%s' "$json" | jq -e '.entries' >/dev/null 2>&1; then
        echo "ask-refactoring: cargo crap failed (exit $rc) or wrote no JSON" >&2
        exit 1
    fi
    printf '%s' "$json"
}

# ── the dedup, decided before the thresholds ────────────────────────────────
#
# Two arms reading one lcov at two granularities report the same gap twice unless
# one defers, and the re-run guard makes that expensive: a duplicate pair blocks
# the next sweep until both halves close. So this arm skips any function whose
# file the coverage arm reports. Not a heuristic — `coverage_arm_files` is that
# arm's predicate, over the same report in the same sweep.
#
# The remedies nest, which is why the split is clean rather than a compromise: a
# file below the coverage floor gets one task whose work is covering the functions
# inside it, so a function task beside it would be that task's first bullet with a
# stem of its own. Nothing is dropped — above the floor, this arm is the only
# reporter.
#
# The measurement that made it worth building, and the merged-arm alternative it
# rules out, are recorded in
# `truth/tasks/sweep-undertested-functions-in-covered-files.typ`. The short of it:
# at this band 6 of today's 10 findings are in files the coverage arm never
# reports, and 4 are duplicates this skip removes.

# ── the band edges, from the real distribution ──────────────────────────────
#
# On today's tree, 1025 production functions are scored (1052 from cargo crap,
# less the 27 the two filters above drop).
#
# coverage < 80, and why not the 88 the file arm uses:
#
#   Per-function coverage has mean 97.61, sd 7.97, with 868 of the 1025 functions
#   at exactly 100%. 80 is ~2.2 sd below the mean — stricter than the coverage
#   arm's 88 (~1.5 sd on the per-file distribution), on purpose. A function is
#   small enough that one unreached line moves its percentage by tens of points,
#   so at this scale the number is exact but grainy: the 80..89 band holds 63
#   functions, and reading them is the argument — `confirm_retire` (88.9%) was
#   fully tested one commit ago, `eval_bool` (87.5%) is a 14-branch dispatcher
#   with one arm unhit. Below 80 sit 28 functions in the whole workspace. That is
#   a tail; the band above it is scatter.
#
# cyclomatic >= 5, and what it rejects:
#
#   CC has mean 3.54, sd 3.05, and 293 of the 1025 functions are at 5 or above,
#   so the floor is not rare by itself — the conjunction is. It is not a
#   complexity judgment either (`just crap` gates that, at 30 and against a
#   committed baseline); it is what makes the coverage number mean something. The
#   18 functions at CC < 5 and coverage < 80 are the noise it drops:
#   `HyperlinkChoice::fmt` and `ColorChoice::fmt` (Display impls at 0% and
#   66.7%), `assert_child_ok` in `test_util.rs` (a test helper crap's globs miss),
#   `set_frameworks` (CC 1 at 66.7% — two of three lines). At CC 1 a percentage is
#   a statement about one branch and the remedy is never a test worth writing.
#   Below the floor a finding says "this line is unhit"; at or above it, "these
#   decisions are unexercised".
#
#   It stops at 5 rather than 6 because 6 drops the four CC-5 findings
#   (`json_to_input_value` at 57%, `Framework::validate` at 60%, two at 72.7%) and
#   takes the yield from 6 unique findings to 2 — not enough to carry an arm.
score_undertested() {
    local reported="$1"
    jq -r \
        --arg repo "$repo/" \
        --arg re "$COVERAGE_TEST_RE" \
        --arg stub "$COVERAGE_STUB" \
        --arg reported "$reported" \
        --argjson cc_floor "$CC_FLOOR" \
        --argjson cov_floor "$FN_COVERAGE_FLOOR" '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      ($reported | split("\n") | map(select(length > 0)))  as $filed
      | .entries[]
      | (.file | ltrimstr($repo))                          as $path
      | select(($path | test($re)) | not)
      | select($path != $stub)
      | select(($filed | index($path)) == null)
      | select(.cyclomatic >= $cc_floor and .coverage < $cov_floor)
      | (.cyclomatic * (1 - .coverage / 100))              as $paths
      | ((1.5 * $paths) | round | clamp(1; 10))            as $impact
      | ((9 - ($paths | round)) | clamp(1; 10))            as $ease
      | (if $paths <= 2 then 2 elif $paths <= 4 then 4 else 6 end) as $difficulty
      | [
          $path,
          .function,
          (.cyclomatic | floor | tostring),
          ((.coverage * 100 | round) / 100 | tostring),
          ($cc_floor | tostring),
          ($cov_floor | tostring),
          ($impact | tostring),
          "0.85",
          ($ease | tostring),
          ($difficulty | tostring)
        ]
      | @tsv
    '
}

# Every number below is driven by one derived quantity, `paths = CC * (1 - cov)`
# — how many of the function's independent paths the suite never walks. That is
# the conjunction this arm bands on, so scoring on it keeps the priority and the
# finding talking about the same thing. On today's tree it spans 1.36
# (`enter_screen`, CC 5 at 72.7%) to 4.8 (`hval_kind`, CC 8 at 40%).
#
# impact = clamp(round(1.5 * paths), 1, 10)
#   The 1.5 spreads the measured range over 2..7 instead of bunching it in the
#   bottom third, and leaves the top for findings this tree does not have: a CC 14
#   dispatcher at 30% scores 10, which is what a 10 should mean. Unexercised paths,
#   not raw complexity and not the raw gap — a CC 10 function at 73%
#   (`unlink_edits`, 2.68) is a smaller problem than a CC 8 one at 40% (4.8).
#
# confidence = 0.85, constant
#   llvm-cov counted the lines and cargo-crap the branches; neither is a forecast.
#   Below the coverage arm's 0.9 for the graininess the band edges discuss — "40%
#   covered" is an exact number about a coarse measurement — plus the same
#   remaining judgment every arm carries, since the remedy can be an exclusion.
#
# ease = clamp(9 - round(paths), 1, 10)
#   Falls with the number of paths a test author must construct, the literal
#   amount of new test material, same shape the coverage arm gets from its
#   uncovered-line count. Deliberately impact's quantity inverted: what makes the
#   gap matter is what makes it work.
#
# difficulty: 2 for <=2 unexercised paths, 4 for <=4, else 6, on the 0-9 judgment
#   scale — the three-tier rubric the other arms use. One or two branches is one
#   test case; five is a fixture that can reach five states.
#
# estimate is a flat "(src-files: 1)": the file's test module gains cases, or
#   `CRAP_EXCLUDES` gains an entry. Both touch one file, so a varying number here
#   would be invented rather than measured.
signal_undertested() {
    ensure_lcov_fresh
    # Captured before the report is scored, not while: `coverage_arm_files` runs
    # `cargo llvm-cov report` a second time in this sweep (arms cannot share a
    # cache — each runs in its own command substitution), and reading it into a
    # variable first keeps that cost and any failure out of the jq pipeline.
    local reported
    reported="$(coverage_arm_files)"
    undertested_report_json | score_undertested "$reported"
}

emit_undertested() {
    local path="$1" function="$2" complexity="$3" percent="$4"
    local cc_floor="$5" cov_floor="$6"
    local impact="$7" confidence="$8" ease="$9" difficulty="${10}"

    # Stem from the file and the function name, never the line: the mutants arm's
    # rule, for its reason — a function keeps its identity across the edits above
    # it that shift its line, so `--dedup skip` still recognises a finding filed
    # once and closed without the gap ever being covered. Two functions of the
    # same name in different files are two findings and get two stems.
    local stem
    stem="$(stem_for undertested "$path-$function")"

    local args=(
        add "cover $function in $path"
        -t sweep-undertested
        --name "$stem"
        --set "path=$path"
        --set "function=$function"
        --set "complexity=$complexity"
        --set "percent=$percent"
        --set "cc_floor=$cc_floor"
        --set "cov_floor=$cov_floor"
        --set "impact=$impact"
        --set "confidence=$confidence"
        --set "ease=$ease"
        --set "difficulty=$difficulty"
        --set "estimate=(src-files: 1)"
        # `coverage`, not a fourth signal tag. The tag says which advisory tool
        # noticed the finding (see the `[tasks.tags]` comment in the config), and
        # that is llvm-cov for both coverage arms — this one is a granularity of
        # the same signal, not a new one. So the config claim holds unchanged:
        # adding an arm is a script and a template. The stem namespace
        # (`undertested-…` vs `coverage-…`) is what tells the two apart.
        --set 'tags+=swept' --set 'tags+=coverage'
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
                echo "        the stem is taken, so this function was filed before and closed" >&2
                echo "        without its coverage rising — reopen it, or exclude the file in" >&2
                echo "        the Justfile's \`CRAP_EXCLUDES\` with a written reason" >&2
                ;;
            *) echo "$outcome $dest" ;;
        esac
    else
        echo "── would file: $stem  [ice($impact, $confidence, $ease)]"
        "$MT" "${args[@]}" --print
        echo
    fi
}
