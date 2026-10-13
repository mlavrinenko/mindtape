# ask-refactoring: the undertested arm — per-FUNCTION complexity x coverage.
#
# Sourced by `contrib/ask-refactoring.sh`; not executable on its own. It reads
# that script's globals (`$repo`, `$MT`, `$WRITE`, the thresholds) and calls its
# `stem_for`. It owns three functions the driver looks up by name —
# `score_undertested`, `signal_undertested`, `emit_undertested` — reading
# `cargo crap --format json` over the lcov `just cover` writes.
#
# `just cover` gates every production file at 88% line coverage (with 5 or more
# uncovered lines), so this arm owns the finer grain alone: no file it reads is
# under that floor, and there is no other arm to dedupe against.
#
# The reasoning behind this arm's numbers lives here with the code that applies
# them, deliberately: a doc it lived in instead would drift from the constants.

# ── the lcov, and what counts as production ─────────────────────────────────
#
# The lcov `just cover` writes is a build artifact nothing re-derives for free.
# `just crap`'s own comment (see the Justfile) documents what a stale one does:
# a source the lcov never saw has no coverage record at all, so it reads as 0%
# covered rather than as unknown — a phantom failure, not a missing measurement.
# `ensure_lcov_fresh` asks mmz to refresh the lcov, then refuses a missing file
# or one whose cover record differs from the sources, so a stale one is
# refused here for the same reason. `just mmz cover` skips the whole
# instrumented test run when no source's content changed since the last
# `just cover`, whatever a checkout did to the mtimes.
ensure_lcov_fresh() {
    # Never onto stdout, which is the TSV row stream: on an mmz cache miss the
    # test run would parse as findings. Its output goes to a log shown only on
    # failure, and one stderr line says why a cache miss pauses for minutes.
    echo "ask-refactoring: just mmz cover (a cache miss reruns the suite)" >&2
    local log
    log="$(mktemp)"
    if ! just mmz cover >"$log" 2>&1; then
        cat "$log" >&2
        rm -f "$log"
        exit 1
    fi
    rm -f "$log"
    # The same script `just crap` asks. It judges by content through mmz, not
    # mtime: a checkout or rebase touches a source without changing its bytes,
    # and mmz then hits and leaves the lcov alone.
    local reason
    reason="$(bash .just/scripts/_lcov-stale-reason.sh)"
    if [ -n "$reason" ]; then
        echo "ask-refactoring: $reason — run \`just cover\` first" >&2
        echo "  (a source the lcov never saw scores as 0% covered, not as unknown)" >&2
        exit 1
    fi
}

# Test files are not production code. `*_tests.rs` / `tests.rs` (and ejectest's
# infix-split `foo_tests_bar.rs` siblings) are instrumented, and `just crap`
# excludes the same files from CRAP scoring (`CRAP_EXCLUDES` in the Justfile).
# The regex is jq's equivalent of those globs, as one pattern because jq has no
# glob matcher. `.just/scripts/cover.sh` holds a copy for its per-file gate.
COVERAGE_TEST_RE='(^|/)([^/]*_)?tests(_[^/]*)?\.rs$'

# ── signal: complexity x coverage, per function ─────────────────────────────
#
# A file-shaped signal is structurally unable to see an undertested function
# inside a well-covered file. `hval_kind` (`crates/mindtape/src/model/mod.rs`) is 40%
# covered at cyclomatic complexity 8, in a file `just cover`'s per-file floor
# passes. Nothing else in the repo reports it; that is this arm's entire reason
# to exist.
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
# Freshness is `ensure_lcov_fresh`, defined below: a source the lcov never saw
# has no record, so `cargo crap` reads it as 0% covered rather than as unknown,
# and a sweep that files phantoms is worse than one that refuses.
#
# Exclusions come from `just _crap-excludes` — the Justfile's `CRAP_EXCLUDES`,
# whose comment there gives the reason a second hand-maintained copy is not
# acceptable; a third in a shell script is the same mistake. Those globs drop
# `main.rs` (absent from the lcov, so every function in it would score 0%) and
# the `*_tests.rs` / `tests.rs` modules. On top of them this arm applies `COVERAGE_TEST_RE`, which catches what
# the globs miss: ejectest's infix-split siblings (`cmd/add/tests_subprocess.rs`).
# `.just/scripts/cover.sh` carries the same regex, so both agree on what
# production code is.
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

# ── the band edges, from the real distribution ──────────────────────────────
#
# On today's tree, 1025 production functions are scored (1052 from cargo crap,
# less the 27 the two filters above drop).
#
# coverage < 80, and why not the 88 `just cover` gates per file:
#
#   Per-function coverage has mean 97.61, sd 7.97, with 868 of the 1025 functions
#   at exactly 100%. 80 is ~2.2 sd below the mean — stricter than the
#   per-file gate's 88 (~1.5 sd on the per-file distribution), on purpose. A function is
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
    jq -r \
        --arg repo "$repo/" \
        --arg re "$COVERAGE_TEST_RE" \
        --argjson cc_floor "$CC_FLOOR" \
        --argjson cov_floor "$FN_COVERAGE_FLOOR" '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      .entries[]
      | (.file | ltrimstr($repo))                          as $path
      | select(($path | test($re)) | not)
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
#   Below 0.9 for the graininess the band edges discuss — "40%
#   covered" is an exact number about a coarse measurement — plus the same
#   remaining judgment every arm carries, since the remedy can be an exclusion.
#
# ease = clamp(9 - round(paths), 1, 10)
#   Falls with the number of paths a test author must construct, the literal
#   amount of new test material, the number of
#   uncovered lines in a file would measure the wrong grain. Deliberately impact's quantity inverted: what makes the
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
    undertested_report_json | score_undertested
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
        # `coverage`, not a further signal tag: the tag says which advisory tool
        # noticed the finding (see the `[tasks.tags]` comment in the config),
        # and that is llvm-cov.
        --set 'tags+=swept' --set 'tags+=coverage'
    )

    local advice; advice=$(printf '%s\n' 'the stem is taken, so this function was filed before and closed' 'without its coverage rising — reopen it, or exclude the file in' 'the Justfile'\''s `CRAP_EXCLUDES` with a written reason')
    file_finding undertested "$stem" "[ice($impact, $confidence, $ease)]" "$advice" "${args[@]}"
}
