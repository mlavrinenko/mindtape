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
#   new homes. Markdown
#   is flat 2: the page and the one it sheds. There is no third file, because
#   nothing has to declare a Markdown page the way a `mod` line declares a
#   module.
#
# The result is monotone in headroom (5,5,4,4,3,3,3 on today's tree) and sits in
# the `normal` band. That is honest: mechanical refactoring is real work and is
# never the most important thing on the board.
#
# Two axes, kept apart deliberately. Language (linecop's own `language` field,
# never re-derived from the extension) decides the remedy: a Rust file is carved
# into a module, a Markdown page sheds a section. Test-versus-production decides
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
# (`kind: "raw"`). This is the language axis and nothing else: what counts as
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

    local tags=(--set 'tags+=swept' --set 'tags+=size')
    [ "$flavour" = test ] && tags+=(--set 'tags+=tests')

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
        --set "estimate=(src-files: $files)"
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
