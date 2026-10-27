# ask-refactoring: the repeats arm — short snippets pasted many times.
#
# Sourced by `contrib/ask-refactoring.sh`; not executable on its own. It reads
# that script's globals (`$repo`, `$MT`, `$WRITE`, `$REPEAT_GAIN`) and calls its
# `stem_for` and `file_finding`. It owns three functions the driver looks up by
# name — `score_repeats`, `signal_repeats`, `emit_repeats` — reading jscpd's
# json report.
#
# `just lint::dry` gates clones of 50 tokens and up, because below that jscpd's
# pairs are too many to gate one by one. What it cannot see is a 15-token line
# pasted 400 times, which this arm reports.
#
# The reasoning behind this arm's numbers lives here with the code that applies
# them, deliberately: a doc it lived in instead would drift from the constants.

# ── signal: jscpd at 15 tokens, grouped ─────────────────────────────────────
#
# Why 15 tokens: it catches a one-line setup such as
# `let dir = tempfile::tempdir().unwrap();`, the most repeated snippet in the
# tree it was chosen on (395 copies, a 17-token window with its neighbours),
# and one line is the smallest thing worth a helper.
#
# Exact tokens, not `--ignore-identifiers`: renamed copies of 50 tokens and up
# are the gate's job.
#
# Grouping: the report's `classes` join the pairs of one snippet, each copy
# once, a copy being a file and the lines it spans. A window that starts a
# line later is another class, which is what the overlap fold below absorbs.
#
# Noise: a fragment of nothing but `use`, `mod`, attribute and bracket lines is
# dropped, and so is one with under 30 characters left once those lines go.
# Those are the shape of a Rust file, not a pasted idea.
#
# Scoring, the tokens an extraction removes, net of what it adds:
#
#   gain = (copies - 1) * tokens - 5 * copies - 10
#
#   Every copy but one goes. Each leaves a call of about 5 tokens, and the
#   helper's own signature costs about 10. So a big block clears the bar with
#   two copies and a one-liner only once it is everywhere, and a common short
#   call such as `foo.bar()` never does, since its gain stays near zero.
#
#   --repeat-gain is the bar, 300 by default: 12 groups on the tree it was
#   chosen on, after the fold. 200 filed 25 and 100 filed 102, a batch nobody
#   empties before the next sweep.
#
# Fold: classes are taken by gain, highest first, and one whose copies mostly
# (half or more) fall on lines a kept class already holds is dropped.
# Without it one setup line files a task per window that starts on it.
#
#   impact = 3 + round(log2(gain / 100)), clamped 1..10
#     Gain spans 300 to several thousand, and doubling it is worth one point.
#
#   confidence = 0.7
#     jscpd measures the repetition, but a repeated shape can be a coincidence,
#     such as two asserts on different types. The remedy is a judgment call.
#
#   ease = 9 - round(files / 10), clamped 1..10
#     Replacing the copies is mechanical, so the cost is how many files it
#     touches.
#
#   difficulty = 2: naming the helper is the one decision, the rest is fan-out.
#   estimate is the files holding a copy, plus one when the helper needs a home.
REPEATS_TOKENS=15

score_repeats() {
    jq -r --argjson min_gain "$REPEAT_GAIN" '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      def noise: test("^\\s*(#!?\\[.*\\]|(pub(\\([a-z]+\\))? )?use .*;|(pub )?mod .*;|[{}()\\];,]*)\\s*$");
      def lines_of($f): [range($f.start; $f.end + 1)] | map("\($f.name):\(.)");
      [.classes[]
       | {fragment, tokens,
          body: ([.fragment | split("\n")[] | select(noise | not)] | join(" ") | gsub("\\s+"; " ")),
          anchor: (.fragments[0] | {name, start}),
          frags: [.fragments[] | {name, start, end}]}]
      | map(select((.body | length) >= 30))
      | map(. + {copies: (.frags | length)})
      | map(. + {gain: ((.copies - 1) * .tokens - 5 * .copies - 10)})
      | map(select(.gain >= $min_gain))
      | sort_by(-.gain, .anchor.name, .anchor.start)
      | reduce .[] as $g ({kept: [], claimed: {}};
          . as $st
          | ([$g.frags[] | select(any(lines_of(.)[]; $st.claimed[.] == true))] | length) as $hit
          | if $hit * 2 >= $g.copies then .
            else .kept += [$g] | .claimed += ([$g.frags[] | lines_of(.)[] | {(.): true}] | add)
            end)
      | .kept[]
      | ([.frags[].name] | unique | length) as $files
      | ((3 + ((.gain / 100) | log2 | round)) | clamp(1; 10)) as $impact
      | ((9 - (($files / 10) | round)) | clamp(1; 10))      as $ease
      | [
          "crates/\(.anchor.name)",
          (.anchor.start | tostring),
          (.copies | tostring),
          (.tokens | tostring),
          (.gain | tostring),
          ($files | tostring),
          .fragment,
          ([.frags | sort_by(.name, .start)[] | "crates/\(.name):\(.start)"] | join("\n")),
          ($impact | tostring),
          ($ease | tostring)
        ]
      | @tsv
    '
}

signal_repeats() {
    local out rc
    out="$(mktemp -d)"
    set +e
    jscpd --no-tips -f rust -k "$REPEATS_TOKENS" -l 1 -r json -o "$out" crates >/dev/null 2>&1
    rc=$?
    set -e
    if ! jq -e '.duplicates | type == "array"' "$out/jscpd-report.json" >/dev/null 2>&1; then
        rm -rf "$out"
        echo "ask-refactoring: jscpd failed (exit $rc) or wrote no json report" >&2
        exit 1
    fi
    score_repeats <"$out/jscpd-report.json"
    rm -rf "$out"
}

emit_repeats() {
    local path="$1" line="$2" copies="$3" tokens="$4" gain="$5" files="$6"
    local fragment="$7" locations="$8" impact="$9" ease="${10}"

    # @tsv escaped the newlines.
    fragment="$(printf '%b' "$fragment")"
    locations="$(printf '%b' "$locations")"

    # Stem from the first copy's file and its text, never the line: an edit
    # above the snippet shifts its line, and `--dedup skip` should still know
    # the finding. The text hash keeps two snippets of one file apart.
    local stem
    stem="$(stem_for repeats "$path-$(printf '%s' "$fragment" | sha1sum | cut -c1-8)")"

    local shown more listed
    shown="$(printf '%s\n' "$locations" | head -n 12 | sed 's/.*/  - #raw("&")/')"
    more=$((copies - 12))
    listed="$shown"
    [ "$more" -gt 0 ] && listed="$shown
  - and $more more, which \`just ask-refactoring\` lists again on its next run"

    local measured="+ 🔴 This snippet repeats $copies times across $files $([ "$files" -eq 1 ] && echo file || echo files), $tokens jscpd tokens each. Extracting it saves about $gain tokens net of the helper and its calls.
$listed"

    local remedy="- \`$path\` and every other file the Problem lists: extract a helper and call it from each copy, or mark the copies as intended with \`// jscpd:ignore-start\` and \`// jscpd:ignore-end\` and a comment saying why the repetition is a coincidence."

    local args=(
        add "extract the snippet $path:$line repeats $copies times"
        -t sweep-repeats
        --name "$stem"
        --set "measured=$measured"
        --set "snippet=$(printf '```rust\n%s\n```' "$fragment")"
        --set "remedy=$remedy"
        --set "impact=$impact"
        --set "confidence=0.7"
        --set "ease=$ease"
        --set "difficulty=2"
        --set "estimate=(src-files: $((files + 1)))"
        --set 'tags+=swept' --set 'tags+=repeats'
    )
    local advice; advice=$(printf '%s\n' 'the stem is taken, so this snippet was filed before and closed' 'without the repetition going — reopen it, or mark the copies with' '`jscpd:ignore-start` and a comment saying why')
    file_finding repeats "$stem" "[$copies copies, gain $gain]" "$advice" "${args[@]}"
}
