# ask-refactoring: the size arm — linecop file sizes at --baseline.
#
# Sourced by `contrib/ask-refactoring.sh`; not executable on its own. It reads
# that script's globals (`$repo`, `$MT`, `$WRITE`, the thresholds) and calls its
# `stem_for`. It owns three functions the driver looks up by name —
# `score_size`, `signal_size`, `emit_size` — reading linecop --format json.
#
# The reasoning behind this arm's numbers lives here with the code that
# applies them, deliberately: it is the most valuable thing in the arm and a
# doc it lived in instead would drift away from the constants it explains.

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
#     High because linecop measures: the line count is a fact, not a forecast.
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
#   estimate is the split path, in `src-files`: the file, the module it sheds,
#   the parent that declares it — 4 when the carve is big enough to want two
#   new homes. Prose is flat 2: the page and the one it sheds. There is no
#   third file, because nothing declares a page the way a `mod` line declares
#   a module, and a render is regenerated, not written.
#
# A test module is reported only from 97% of its cap. The first sweep's lanes
# split six test files at 90..95% and called it busywork: a test file grows by
# whole cases, its seams are arbitrary, and nothing reads it end to end.
#
# The result is monotone in headroom (5,5,4,4,3,3,3 on today's tree) and sits in
# the `normal` band. That is honest: mechanical refactoring is real work and is
# never the most important thing on the board.
#
# Two axes, kept apart deliberately. Kind decides the remedy: linecop's own
# `language` field, never re-derived from the extension, except that prose
# (Markdown, or a doc source `size_meta` marks) is one kind whatever its
# language. A Rust file is carved into a module, prose is rewritten.
# Test-versus-production decides
# tone and score, and is a Rust distinction only — it is what `ejectest` already
# played its move on — so it is scored `false` for anything else rather than
# left to a path regex that would happen to agree.
# A rendered doc and its source are one artifact, so they make one finding.
# `size_meta` maps each reported path to what `score_size` folds by: the
# source a render names in its `Generated from` banner, and whether the file is
# prose (a render, or a Typst source carrying a `<mindtape-md>` block). It
# reads linecop's JSON on stdin and prints `{path: {source?, prose?}}`.
size_meta() {
    local path src
    jq -r '.[].path | ltrimstr("./")' | while read -r path; do
        src="$(sed -n 's/^<!-- Generated from \(.*\) by `just docs md`\..*/\1/p' "$path" 2>/dev/null | head -n 1)"
        if [ -n "$src" ]; then
            jq -n --arg p "$path" --arg s "$src" '{($p): {source: $s, prose: true}}'
        elif grep -q '<mindtape-md>' "$path" 2>/dev/null; then
            jq -n --arg p "$path" '{($p): {prose: true}}'
        fi
    done | jq -s 'add // {}'
}

# `$1` is `size_meta`'s map, `{}` when absent. Rows are grouped by source, the
# tightest member scores the group, and `measured` holds one Problem line per
# member, with newlines escaped by @tsv.
score_size() {
    local meta="${1:-}"
    [ -n "$meta" ] || meta='{}'
    jq -r --argjson meta "$meta" '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      def pct: (.lines / .limit * 1000 | round) / 10;
      map((.path | ltrimstr("./")) as $p
          | ($meta[$p] // {}) as $m
          | . + {path: $p, source: ($m.source // $p),
                 prose: ($m.prose // (.language == "Markdown"))})
      | group_by(.source)[]
      | sort_by(-(.lines / .limit)) as $members
      | $members[0]                                         as $top
      | $top.source                                         as $path
      | (if any($members[]; .prose) then "prose" else $top.language end) as $kind
      | ($kind == "Rust"
         and ($path | test("_tests\\.rs$|(^|/)tests\\.rs$|(^|/)tests_[^/]*\\.rs$|(^|/)tests/"))) as $is_test
      | select(($is_test | not) or ($top | pct) >= 97)
      | (100 - ($top | pct))                                as $free
      | ([$top.lines - $top["baseline-limit"], 0] | max)    as $move
      | ((10 - (($free / 1.5) | round)) | clamp(1; 10))     as $impact
      | (if $is_test then 0.7 else 0.9 end)                 as $confidence
      | ((9 - (($move / 20) | round)) - (if $is_test then 2 else 0 end) | clamp(1; 10)) as $ease
      | ((if $move <= 20 then 3 elif $move <= 40 then 4 else 5 end)
          + (if $is_test then 1 else 0 end))                as $difficulty
      | (if $kind == "prose" then 2
         elif $move > 40 then 4 else 3 end)                 as $files
      | [
          $path,
          $kind,
          ([$members[] | "+ 🔴 #raw(\"\(.path)\") stands at \(.lines) of its \(.limit)-line budget: \(.limit - .lines) lines of headroom left, \(pct)% of the cap spent."] | join("\n")),
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
    printf '%s' "$json" | score_size "$(printf '%s' "$json" | size_meta)"
}

# One plain sentence on where the seam question stands. Rust splits on the
# test-versus-production axis; every other language has only the one form,
# because ejectest is a Rust tool and the distinction it draws does not exist
# outside it. Plain prose, no backticks: the `seam` slot is `kind: "text"`, so
# markup would be escaped and shown literally.
seam_for() {
    case "$1:$2" in
        Rust:test) echo "Test code: ejectest already moved these tests out of their parent once, so the eject recipe has no second move here and any further seam is one you draw by hand." ;;
        Rust:*) echo "Production code: linecop measures the file, it cannot tell you where the file wants to be cut, so the seam is yours to choose." ;;
        prose:*) echo "Prose: the seam is a section boundary, and the question is which sections a reader needs in front of them every time versus which ones they go looking for." ;;
        *) echo "linecop measures the file, it cannot tell you where the file wants to be cut, so the seam is yours to choose." ;;
    esac
}

# The first two Scope bullets, spliced into the `remedy` slot as real markup
# (`kind: "raw"`). This is the kind axis and nothing else: what counts as
# a remedy here, and what a split has to amount to for it to have been one. The
# bullets after it in the template are kind-neutral and stay literal there.
#
# Prose is a different remedy, not the Rust one reworded. Its rule lives once,
# in docs/contributing/documentation.md, so this links to it.
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
        prose)
            cat <<'EOF'
- Follow "Rewrite the whole file" in `docs/contributing/documentation.md`:
  rewrite the source whole for its reader, or move an argument where a reader
  reaches it deliberately, such as a deep dive or a skill's `references/`.
  Trimming words to fit is not a fix.
- Re-cap only a file no session loads whole. The `.linecop.yaml` override of an
  always-loaded file says it is rewritten, never raised.
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
    local path="$1" kind="$2" measured="$3" flavour="$4"
    local impact="$5" confidence="$6" ease="$7" difficulty="$8" files="$9"

    local title="split or re-cap $path"
    [ "$kind" = prose ] && title="rewrite $path under its cap"
    # @tsv escaped the newlines between measured lines.
    measured="$(printf '%b' "$measured")"

    local seam remedy
    seam="$(seam_for "$kind" "$flavour")"
    # Trailing newline stripped by the substitution, so the spliced bullets abut
    # the literal ones after the slot and the Scope reads as one list.
    remedy="$(remedy_for "$kind")"

    local tags=(--set 'tags+=swept' --set 'tags+=size')
    [ "$flavour" = test ] && tags+=(--set 'tags+=tests')

    local args=(
        add "$title"
        -t sweep-size
        --name "$(stem_for size "$path")"
        --set "measured=$measured"
        --set "seam=$seam"
        --set "remedy=$remedy"
        --set "impact=$impact"
        --set "confidence=$confidence"
        --set "ease=$ease"
        --set "difficulty=$difficulty"
        --set "estimate=(src-files: $files)"
        "${tags[@]}"
    )

    local advice; advice=$(printf '%s\n' 'the stem is taken, so this finding was filed before and closed' 'without the file shrinking — reopen it, or write the override')
    file_finding size "$(stem_for size "$path")" "[$kind $flavour, ice($impact, $confidence, $ease)]" "$advice" "${args[@]}"
}
