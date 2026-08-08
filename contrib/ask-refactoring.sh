#!/usr/bin/env bash
# ask-refactoring.sh — turn advisory quality signals into backlog tasks.
#
# Run it as `just ask-refactoring` (dry) or `just ask-refactoring --write`.
#
# Mechanical refactoring work recurs, and today a human notices it at the moment
# it blocks something else. The tools that predict it are already
# machine-readable; nothing reads them into the backlog. This does.
#
# Three rules the design turns on:
#
#   Only ADVISORY signals qualify. A signal that already blocks a commit —
#   jscpd, outdatty, machete, clippy — must never file a task, because the wall
#   is already in front of you and the task would be a note about the wall.
#
#   One task per FINDING, never one per sweep. Each is dispatched to its own
#   agent and has to carry its own scope.
#
#   Idempotency comes from the SIGNAL, not from a ledger. Nothing here records
#   what was filed. A closed task whose condition still holds should be noticed
#   again — the work did not happen. The one deliberate suppression is an
#   override with a written reason in the signal's own config (.linecop.yaml for
#   sizes; an `--exclude` entry in the `just cover`/`just crap` recipes, with the
#   reason in the comment beside it, for coverage), which is the mechanism those
#   files already use.
#
# The re-run guard is the [views.ask-refactoring] view in .mindtape/config.toml:
# while any task the sweep filed is still open, the sweep refuses. One sprint of
# mechanical work is emptied before the next is filed.
#
# Adding a signal is a scoring function plus a template, not a config change:
# the `swept` / `size` / `mutants` / `coverage` tags are all declared already,
# and the guard selects on `swept` alone. Signals live in `signal_<name>`
# functions that print TSV rows on stdout; `emit` turns a row into a task.
#
# NOT memoized with mmz, deliberately: the whole point is to re-observe the
# tree on every run. The one exception is `just cover` itself, inside the
# coverage signal: `just mmz cover` (see below) skips re-running the whole test
# suite when nothing under its scope changed, but the lcov it reads is still
# re-checked for freshness on every sweep — memoizing the expensive rebuild is
# not the same as memoizing the observation.
set -eo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo"

BASELINE=90
COVERAGE_FLOOR=88
WRITE=0

usage() {
    cat <<'EOF'
usage: just ask-refactoring [--write] [--baseline PCT] [--coverage-floor PCT]

  --write               file the tasks (default: print what would be filed)
  --baseline PCT        report files at or above PCT% of their linecop limit [90]
  --coverage-floor PCT  report files below PCT% line coverage [88]

env:
  MT   path to the `mt` binary (default: build and use ./target/debug/mt)
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --write) WRITE=1; shift ;;
        --baseline) BASELINE="$2"; shift 2 ;;
        --baseline=*) BASELINE="${1#*=}"; shift ;;
        --coverage-floor) COVERAGE_FLOOR="$2"; shift 2 ;;
        --coverage-floor=*) COVERAGE_FLOOR="${1#*=}"; shift ;;
        -h | --help) usage; exit 0 ;;
        *) echo "ask-refactoring: unknown argument: $1" >&2; usage >&2; exit 2 ;;
    esac
done

# Use the worktree's own `mt`, never whatever is on PATH: the sweep depends on
# `--dedup` and `--format json`, which a system-wide install may predate, and a
# tracker that files tasks with a stale binary is worse than one that does not
# run. Same convention as `just check-doc-coverage`.
MT="${MT:-}"
if [ -z "$MT" ]; then
    cargo build -p mindtape-cli -q
    MT="$repo/target/debug/mt"
fi

# ── the re-run guard ────────────────────────────────────────────────────────
open="$("$MT" ls '@ask-refactoring' --format path)"
if [ -n "$open" ]; then
    count="$(printf '%s\n' "$open" | grep -c .)"
    echo "ask-refactoring: $count task(s) from the last sweep are still open:" >&2
    printf '%s\n' "$open" | sed 's/^/  /' >&2
    cat >&2 <<'EOF'

Empty the view before sweeping again — finish them, cancel them, or write the
override that makes the finding stop existing. Filing a second sprint on top of
the first is how a signal-driven backlog becomes noise.

  mt ls @ask-refactoring
EOF
    exit 1
fi

# ── signal: linecop ─────────────────────────────────────────────────────────
#
# Scoring, and why these numbers and not a constant triple:
#
#   impact = 10 - round(free% / 1.5), clamped 1..10
#     `free%` is how much of the cap is still unspent. A file at its limit
#     blocks the next edit outright; one at 91% has forty-odd lines of room and
#     can wait. One impact point per 1.5% of the cap keeps the whole reported
#     band (85%..100%) spread across the scale instead of bunched at the top.
#
#   confidence = 0.9 production, 0.7 test
#     High because linecop MEASURES: the line count is a fact, not a forecast.
#     Not 1.0 because the remedy is a judgment call — a re-cap with a reason is
#     as valid an outcome as a split. Test modules score lower still: ejectest
#     already played its one move on them, so whether a further split is even
#     the right answer is genuinely open.
#
#   ease = 9 - round(move / 20), minus 2 for a test module, clamped 1..10
#     `move` is how many lines have to find a new home to get back under the
#     baseline. The remedy is well understood, so ease starts high; the only
#     real cost is volume. Test modules lose two: `just eject` has no second
#     move on an already-ejected `*_tests.rs`, so the seam is drawn by hand.
#
#   difficulty tracks `move` on the config's judgment rubric (3 = one obvious
#   extraction, 5 = several defensible seams), +1 for a test module.
#
#   estimate is the split path: the file, the module it sheds, the parent that
#   declares it — 4 when the carve is big enough to want two new homes. Markdown
#   is flat 2: the page and the one it sheds. There is no third file, because
#   nothing has to declare a Markdown page the way a `mod` line declares a
#   module.
#
# The result is monotone in headroom (5,5,4,4,3,3,3 on today's tree) and sits in
# the `normal` band. That is honest: mechanical refactoring is real work and is
# never the most important thing on the board.
#
# Two axes, kept apart deliberately. LANGUAGE (linecop's own `language` field,
# never re-derived from the extension) decides the remedy: a Rust file is carved
# into a module, a Markdown page sheds a section. TEST-VERSUS-PRODUCTION decides
# tone and score, and is a Rust distinction only — it is what `ejectest` already
# played its move on — so it is scored `false` for anything else rather than
# left to a path regex that would happen to agree.
score_size() {
    jq -r '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      .[]
      | (.path | ltrimstr("./"))                            as $path
      | (.language)                                         as $language
      | ($language == "Rust"
         and ($path | test("_tests\\.rs$|(^|/)tests\\.rs$|(^|/)tests/"))) as $is_test
      | (.lines / .limit * 100)                             as $pct
      | (100 - $pct)                                        as $free
      | (.limit - .lines)                                   as $headroom
      | ([.lines - .["baseline-limit"], 0] | max)           as $move
      | ((10 - (($free / 1.5) | round)) | clamp(1; 10))     as $impact
      | (if $is_test then 0.7 else 0.9 end)                 as $confidence
      | ((9 - (($move / 20) | round)) - (if $is_test then 2 else 0 end) | clamp(1; 10)) as $ease
      | ((if $move <= 20 then 3 elif $move <= 40 then 4 else 5 end)
          + (if $is_test then 1 else 0 end))                as $difficulty
      | (if $language == "Markdown" then 2
         elif $move > 40 then 4 else 3 end)                 as $files
      | [
          $path,
          $language,
          (.lines | tostring),
          (.limit | tostring),
          ($headroom | tostring),
          (($pct * 10 | round) / 10 | tostring),
          (if $is_test then "test" else "production" end),
          ($impact | tostring),
          ($confidence | tostring),
          ($ease | tostring),
          ($difficulty | tostring),
          ($files | tostring)
        ]
      | @tsv
    '
}

signal_size() {
    local json rc
    set +e
    json="$(linecop --baseline "$BASELINE" --format json)"
    rc=$?
    set -e
    # linecop exits 1 when it reports files and 0 when it reports none, so a
    # non-zero exit is not an error here. Anything that is not a JSON array is.
    if ! printf '%s' "$json" | jq -e 'type == "array"' >/dev/null 2>&1; then
        echo "ask-refactoring: linecop failed (exit $rc) or wrote no JSON" >&2
        exit 1
    fi
    printf '%s' "$json" | score_size
}

# A stem derived from the finding, not from the title: stable across runs, so
# `--dedup skip` recognises the same finding twice, and namespaced by signal so
# the three arms cannot collide. `$1` is the signal name (`size`, `coverage`,
# ...), `$2` the finding's path.
stem_for() { printf '%s-%s\n' "$1" "$(printf '%s' "$2" | tr -c 'A-Za-z0-9' '-' | tr -s '-' | sed 's/^-//; s/-$//')"; }

# One plain sentence on where the seam question stands. Rust splits on the
# test-versus-production axis; every other language has only the one form,
# because ejectest is a Rust tool and the distinction it draws does not exist
# outside it. Plain prose, no backticks: the `seam` slot is `kind: "text"`, so
# markup would be escaped and shown literally.
seam_for() {
    case "$1:$2" in
        Rust:test) echo "Test code: ejectest already moved these tests out of their parent once, so the eject recipe has no second move here and any further seam is one you draw by hand." ;;
        Rust:*) echo "Production code: linecop measures the file, it cannot tell you where the file wants to be cut, so the seam is yours to choose." ;;
        Markdown:*) echo "Prose: the seam is a section boundary, and the question is which sections a reader needs in front of them every time versus which ones they go looking for." ;;
        *) echo "linecop measures the file, it cannot tell you where the file wants to be cut, so the seam is yours to choose." ;;
    esac
}

# The first two Scope bullets, spliced into the `remedy` slot as real markup
# (`kind: "typst"`). This is the language axis and nothing else: what counts as
# a remedy here, and what a split has to amount to for it to have been one. The
# bullets after it in the template are language-neutral and stay literal there.
#
# Markdown is genuinely a different remedy, not the Rust one reworded. The cost
# of the wrong answer — compressing in place until the budget goes on phrasing
# instead of on being right — is already argued at length next to the 300-line
# overrides in .linecop.yaml, so this points at that reasoning rather than
# restating it.
remedy_for() {
    case "$1" in
        Rust)
            cat <<'EOF'
- Pick the remedy, then do it. Either carve the file along a seam that means
  something, or raise its cap in `.linecop.yaml` with an override that names a
  new sane number and says why. Both are correct answers. A split for its own
  sake is not: the cap is there to push back on scope growth, not on accuracy.
- If you split, the extracted module has to stand on its own and be named for
  what it does. An arbitrary bottom half moved into a `_more.rs` clears linecop
  and misses the point entirely.
EOF
            ;;
        Markdown)
            cat <<'EOF'
- Pick the remedy, then do it. Either move a section off the always-loaded
  surface, or raise its cap in `.linecop.yaml` with an override that names a new
  sane number and says why. Both are correct answers. Compressing the prose in
  place is not, and `.linecop.yaml` already records what it costs: the 300-line
  overrides exist because a wall mid-edit turned a round of correctness fixes
  into a dozen rewording passes.
- If you move a section, move it somewhere a reader reaches deliberately —
  progressive disclosure into `references/` for a skill, its own page for the
  docs site. What the cap defends is the surface that loads every time, so a
  section still pulled in from a second file has not actually moved.
EOF
            ;;
        *)
            cat <<'EOF'
- Pick the remedy, then do it. Either move a self-contained part of this file
  somewhere it stands on its own, or raise its cap in `.linecop.yaml` with an
  override that names a new sane number and says why. Both are correct answers.
  A split for its own sake is not: the cap is there to push back on scope
  growth, not on accuracy.
- If you split, what comes out has to be nameable for what it does. A cut made
  where the line count happens to land clears linecop and misses the point.
EOF
            ;;
    esac
}

emit_size() {
    local path="$1" language="$2" lines="$3" limit="$4" headroom="$5" pct="$6" flavour="$7"
    local impact="$8" confidence="$9" ease="${10}" difficulty="${11}" files="${12}"

    local seam remedy
    seam="$(seam_for "$language" "$flavour")"
    # Trailing newline stripped by the substitution, so the spliced bullets abut
    # the literal ones after the slot and the Scope reads as one list.
    remedy="$(remedy_for "$language")"

    local tags=(--set 'tags[]=swept' --set 'tags[]=size')
    [ "$flavour" = test ] && tags+=(--set 'tags[]=tests')

    local args=(
        add "split or re-cap $path"
        -t sweep-size
        --name "$(stem_for size "$path")"
        --set "path=$path"
        --set "lines=$lines"
        --set "limit=$limit"
        --set "headroom=$headroom"
        --set "percent=$pct"
        --set "seam=$seam"
        --set "remedy=$remedy"
        --set "impact=$impact"
        --set "confidence=$confidence"
        --set "ease=$ease"
        --set "difficulty=$difficulty"
        --set "estimate=(files: $files)"
        "${tags[@]}"
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
                echo "        without the file shrinking — reopen it, or write the override" >&2
                ;;
            *) echo "$outcome $dest" ;;
        esac
    else
        echo "── would file: $(stem_for size "$path")  [$language $flavour, ice($impact, $confidence, $ease)]"
        "$MT" "${args[@]}" --print
        echo
    fi
}

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
    just mmz cover
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
score_coverage() {
    jq -r --arg repo "$repo/" --argjson floor "$COVERAGE_FLOOR" '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      .data[0].files[]
      | select(.summary.lines.count > 0)
      | (.filename | ltrimstr($repo))                       as $path
      | select(($path | test("(^|/)([^/]*_)?tests(_[^/]*)?\\.rs$")) | not)
      | select($path != "crates/mindtape/src/recur_rrule_off.rs")
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
signal_coverage() {
    ensure_lcov_fresh
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
    printf '%s' "$json" | score_coverage
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

# Each signal's rows are captured with `$(...)` first, not read straight off a
# `< <(...)` process substitution: a signal that hits its own `exit 1` (a
# malformed linecop report, a stale lcov) is running in a subshell either way,
# but only the command-substitution form propagates that exit through `set -e`
# to this script. Piped directly into a `read` loop via process substitution,
# the same `exit 1` would kill only the substitution's subshell — the loop
# would just see end-of-input, "0 rows from this signal" would look identical
# to "this signal is refusing to run", and the sweep would report a clean,
# wrong result. Verified with a minimal repro before relying on it here.
findings=0
size_rows="$(signal_size)"
while IFS=$'\t' read -r -a row; do
    [ "${#row[@]}" -gt 0 ] || continue
    emit_size "${row[@]}"
    findings=$((findings + 1))
done <<<"$size_rows"

coverage_rows="$(signal_coverage)"
while IFS=$'\t' read -r -a row; do
    [ "${#row[@]}" -gt 0 ] || continue
    emit_coverage "${row[@]}"
    findings=$((findings + 1))
done <<<"$coverage_rows"

if [ "$findings" -eq 0 ]; then
    echo "ask-refactoring: no findings at --baseline $BASELINE / --coverage-floor $COVERAGE_FLOOR"
elif [ "$WRITE" -eq 0 ]; then
    echo "ask-refactoring: $findings finding(s); re-run with --write to file them"
else
    echo "ask-refactoring: $findings finding(s) swept"
fi
