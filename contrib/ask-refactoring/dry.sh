# ask-refactoring: the dry arm — the clones `just lint::dry` tolerates.
#
# Sourced by `contrib/ask-refactoring.sh`; not executable on its own. It reads
# that script's globals (`$repo`, `$MT`, `$WRITE`, `$DRY_GAIN`) and calls its `stem_for` and
# `file_finding`. It owns three functions the driver looks up by name —
# `score_dry`, `signal_dry` and `emit_dry` — reading jscpd's json report.
#
# The gate stops a new clone, and `.config/jscpd-baseline.json` lists the ones
# that predate it. A tolerated clone is not enforced by anything, so it is the
# part of jscpd this sweep may report (the header of `ask-refactoring.sh` rules
# out only what a gate still enforces). The arm runs jscpd with the gate's own
# `DRY_FLAGS` and keeps the pairs the report marks `isNew: false`.
#
# Grouping: jscpd reports pairs, so a snippet pasted three times is up to three
# of them. The pairs are joined into families by a shared location (file and
# first line), which is jscpd's clone class until upstream ships one
# (github:kucherenko/jscpd#1000, the `jscpd-clone-classes` blocker in
# `.config/canibump.toml`).
#
# Fold: families are taken by gain, highest first, and one whose locations
# mostly (half or more) fall on lines a kept family already holds is dropped.
# A clone that slides by one line would otherwise file a task per window.
#
# Families anchored in one file make one task: jscpd reports a pattern's
# sliding windows as separate families, and the first sweep filed the same
# assert helper as five tasks. Their gains add up; each family clears the bar
# on its own first.
#
# The bar is `--dry-gain`, 300 by default, the repeats arm's number. With no
# bar the arm reported 639 families at a median gain of 50 tokens, 510 of them
# in tests: a batch nobody empties. 100 still left 147; 300 leaves 25, each a
# block worth a helper.
#
#   gain = (copies - 1) * tokens - 5 * copies - 10
#     The tokens an extraction removes, net of a 5-token call per copy and a
#     10-token helper, with the family's largest pair as `tokens`.
#
#   impact = 3 + round(log2(gain / 100)), clamped 1..10
#     Doubling the gain is worth one point. Most tolerated clones are a pair
#     of 50 to 100 tokens, so most land at 1 or 2.
#
#   confidence = 0.6
#     The detector ignores names and literals, so a match can be a coincidence
#     such as a lookup table matching itself one row down.
#
#   ease = 9 - round(files / 10), clamped 1..10, difficulty = 2
#     Naming the helper is the one decision, the rest is fan-out.
#     estimate is the files holding a copy, plus one when the helper needs a
#     home.

score_dry() {
    jq -r --argjson min_gain "$DRY_GAIN" '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      def loc: "\(.name):\(.start)";
      def lines_of($f): [range($f.start; $f.end + 1)] | map("\($f.name):\(.)");
      def join_pair($p):
        ($p.firstFile | loc) as $a | ($p.secondFile | loc) as $b
        | (.comp[$a]) as $ca | (.comp[$b]) as $cb
        | (if $ca == null and $cb == null then
             (.n + 1) as $id | .n = $id
             | .comp[$a] = $id | .comp[$b] = $id
             | .mem[($id | tostring)] = [$a, $b]
           elif $cb == null then
             .comp[$b] = $ca | .mem[($ca | tostring)] += [$b]
           elif $ca == null then
             .comp[$a] = $cb | .mem[($cb | tostring)] += [$a]
           elif $ca == $cb then .
           else
             (.mem[($cb | tostring)]) as $m
             | reduce $m[] as $x (.; .comp[$x] = $ca)
             | .mem[($ca | tostring)] += $m
             | del(.mem[($cb | tostring)])
           end)
        | .nodes[$a] = ($p.firstFile | {name, start, end})
        | .nodes[$b] = ($p.secondFile | {name, start, end})
        | .pairs[$a] += [$p | {tokens, fragment, anchor: .firstFile.name, start: .firstFile.start}];
      [.duplicates[] | select(.isNew | not)] as $dups
      | reduce $dups[] as $p ({comp: {}, mem: {}, nodes: {}, pairs: {}, n: 0}; join_pair($p))
      | . as $st
      | [$st.mem[]
         | . as $members
         | ([$members[] | $st.pairs[.] // [] | .[]] | sort_by(-.tokens, .anchor, .start) | .[0]) as $best
         | {tokens: $best.tokens, fragment: $best.fragment,
            anchor: {name: $best.anchor, start: $best.start},
            frags: ([$members[] | $st.nodes[.]] | sort_by(.name, .start))}
         | . + {copies: (.frags | length)}
         | . + {gain: ((.copies - 1) * .tokens - 5 * .copies - 10)}]
      | sort_by(-.gain, .anchor.name, .anchor.start)
      | reduce .[] as $g ({kept: [], claimed: {}};
          . as $k
          | ([$g.frags[] | select(any(lines_of(.)[]; $k.claimed[.] == true))] | length) as $hit
          | if $hit * 2 >= $g.copies then .
            else .kept += [$g] | .claimed += ([$g.frags[] | lines_of(.)[] | {(.): true}] | add)
            end)
      | [.kept[] | select(.gain >= $min_gain)]
      | group_by(.anchor.name)[]
      | sort_by(-.gain)
      | (.[0] + {gain: (map(.gain) | add), copies: (map(.copies) | add),
                 families: length, frags: (map(.frags[]) | unique)})
      | ([.frags[].name] | unique | length) as $files
      | ((3 + (([.gain, 1] | max / 100 | log2) | round)) | clamp(1; 10)) as $impact
      | ((9 - (($files / 10) | round)) | clamp(1; 10)) as $ease
      | [
          "crates/\(.anchor.name)",
          (.anchor.start | tostring),
          (.copies | tostring),
          (.tokens | tostring),
          (.gain | tostring),
          ($files | tostring),
          .fragment,
          ([.frags[] | "crates/\(.name):\(.start)"] | join("\n")),
          ($impact | tostring),
          ($ease | tostring),
          (.families | tostring)
        ]
      | @tsv
    '
}

signal_dry() {
    local out rc flags
    out="$(mktemp -d)"
    # The gate's own flags, so the arm and the gate agree on what a clone is
    # and which ones the baseline tolerates.
    flags="$(just --justfile .just/lint.just --evaluate DRY_FLAGS)"
    set +e
    # shellcheck disable=SC2086
    jscpd $flags -r json -o "$out" crates >/dev/null 2>&1
    rc=$?
    set -e
    if ! jq -e '.duplicates | type == "array"' "$out/jscpd-report.json" >/dev/null 2>&1; then
        rm -rf "$out"
        echo "ask-refactoring: jscpd failed (exit $rc) or wrote no json report" >&2
        exit 1
    fi
    score_dry <"$out/jscpd-report.json"
    rm -rf "$out"
}

emit_dry() {
    local path="$1" line="$2" copies="$3" tokens="$4" gain="$5" files="$6"
    local fragment="$7" locations="$8" impact="$9" ease="${10}" families="${11}"

    # @tsv escaped the newlines.
    fragment="$(printf '%b' "$fragment" | head -n 8)"
    locations="$(printf '%b' "$locations")"

    # Stem from the anchor's file and its text, never the line: an edit above
    # the clone shifts its line, and `--dedup skip` should still know it.
    local stem
    stem="$(stem_for dry "$path-$(printf '%s' "$fragment" | sha1sum | cut -c1-8)")"

    local shown more listed
    shown="$(printf '%s\n' "$locations" | head -n 6 | sed 's/.*/  - #raw("&")/')"
    more=$((copies - 6))
    listed="$shown"
    [ "$more" -gt 0 ] && listed="$shown
  - and $more more, which \`just ask-refactoring\` lists again on its next run"

    local shape="This clone appears"
    [ "$families" -gt 1 ] && shape="$families clones anchored in this file appear"
    local measured="+ 🔴 $shape $copies times across $files $([ "$files" -eq 1 ] && echo file || echo files), $tokens jscpd tokens at most. Extracting it saves about $gain tokens net of the helper and its calls.
$listed"

    local remedy="- \`$path\` and every other file the Problem lists: extract a helper and call it from each copy, or mark the copies as intended with \`// jscpd:ignore-start\` and \`// jscpd:ignore-end\` and a comment saying why the match is a coincidence. Either way, re-mint the baseline with \`just lint::dry-baseline\` in the same commit."

    local args=(
        add "extract the clone at $path:$line ($copies copies)"
        -t sweep-dry
        --name "$stem"
        --set "measured=$measured"
        --set "snippet=$(printf '```rust\n%s\n```' "$fragment")"
        --set "remedy=$remedy"
        --set "impact=$impact"
        --set "confidence=0.6"
        --set "ease=$ease"
        --set "difficulty=2"
        --set "estimate=(src-files: $((files + 1)))"
        --set 'tags+=swept' --set 'tags+=dry'
    )
    local advice
    advice="$(printf '%s\n' \
        'the stem is taken, so this clone was filed before and closed' \
        'without the baseline losing it — reopen it, or mark the copies with' \
        '`jscpd:ignore-start` and a comment saying why, then `just lint::dry-baseline`')"
    file_finding dry "$stem" "[$copies copies, gain $gain]" "$advice" "${args[@]}"
}
