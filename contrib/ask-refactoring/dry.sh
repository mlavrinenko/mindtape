# ask-refactoring: the dry arm: the clones the reasons file records as debt.
#
# Sourced by `contrib/ask-refactoring.sh`, not executable on its own. It reads
# that script's globals (`$repo`, `$MT`, `$WRITE`) and calls its `stem_for` and
# `file_finding`. It owns three functions the driver looks up by name:
# `score_dry`, `signal_dry` and `emit_dry`, reading jscpd's json report.
#
# The gate stops a new clone, and `.config/jscpd-baseline.json` lists the ones
# that predate it. A tolerated clone is not enforced by anything, so it is the
# part of jscpd this sweep may report (the header of `ask-refactoring.sh` rules
# out only what a gate still enforces). Every tolerated clone carries a reason
# in `.config/jscpd-baseline-reasons.json`, and a reason that ends `is a
# separate task` names an extraction nobody got to: the debt. The arm writes
# those fingerprints to a baseline of its own and runs jscpd with the gate's
# flags against it, so jscpd itself says which pairs are debt. It reads the
# report's `duplicates` pairs, groups them by fragment, and keeps the groups a
# debt pair belongs to.
#
# Fold: gain orders the groups, highest first, and one whose copies (half or
# more) fall on lines a kept group already holds gets dropped. A clone that
# slides by one line would otherwise file a task per window.
#
# No bar: the reasons file is the bar. A clone is debt because a maintainer
# wrote that its extraction is a separate task, not because its gain cleared a
# number. The debt's smallest clone is worth 30 tokens and its largest 135, so
# a threshold would either file nothing or file everything.
#
#   gain = (copies - 1) * tokens - 5 * copies - 10.
#     The tokens an extraction removes, net of a 5-token call per copy and a
#     10-token helper, with the tokens of the group's smallest pair.
#
#   impact = 3 + round(log2(gain / 100)), clamped 1..10
#     Doubling the gain is worth one point. The debt is a pair of 50 to 100
#     tokens, so most land at 1 or 2.
#
#   confidence = 0.6
#     The detector ignores names and literals, so a match can be a coincidence
#     such as a lookup table matching itself one row down.
#
#   ease = 9 - round(files / 10), clamped 1..10, difficulty = 2
#     Naming the helper is the one decision, the rest is fan-out.

score_dry() {
    jq -r '
      def clamp($lo; $hi): [[., $lo] | max, $hi] | min;
      def lines_of($f): [range($f.start; $f.end + 1)] | map("\($f.name):\(.)");
      def key($f): "\($f.name):\($f.start):\($f.end)";
      [.duplicates[]
       | {fragment, tokens, debt: (.isNew | not),
          frags: ([.firstFile, .secondFile] | map({name, start, end}))}]
      | group_by(.fragment)
      | map({fragment: .[0].fragment,
             tokens: ([.[].tokens] | min),
             debt: ([.[].debt] | any),
             frags: ([.[].frags[]] | unique_by(key(.)) | sort_by(.name, .start))})
      | map(select(.debt))
      | map(. + {copies: (.frags | length)})
      | map(. + {gain: ((.copies - 1) * .tokens - 5 * .copies - 10)})
      | sort_by(-.gain, .frags[0].name, .frags[0].start)
      | reduce .[] as $g ({kept: [], claimed: {}};
          . as $k
          | ([$g.frags[] | select(any(lines_of(.)[]; $k.claimed[.] == true))] | length) as $hit
          | if $hit * 2 >= $g.copies then .
            else .kept += [$g] | .claimed += ([$g.frags[] | lines_of(.)[] | {(.): true}] | add)
            end)
      | .kept[]
      | ([.frags[].name] | unique | length) as $files
      | ((3 + (([.gain, 1] | max / 100 | log2) | round)) | clamp(1; 10)) as $impact
      | ((9 - (($files / 10) | round)) | clamp(1; 10)) as $ease
      | [
          "crates/\(.frags[0].name)",
          (.frags[0].start | tostring),
          (.copies | tostring),
          (.tokens | tostring),
          (.gain | tostring),
          ($files | tostring),
          .fragment,
          ([.frags[] | "crates/\(.name):\(.start)"] | join("\n")),
          ($impact | tostring),
          ($ease | tostring)
        ]
      | @tsv
    '
}

signal_dry() {
    local out rc flags debt
    out="$(mktemp -d)"
    debt="$out/debt-baseline.json"
    # The reasons file records the debt. jscpd keys a clone by a fingerprint of
    # its tokens, and the report does not carry that fingerprint. The arm hands
    # jscpd the debt's fingerprints as a baseline. jscpd then says which pairs
    # they are.
    jq '{version: 1,
         fingerprints: (with_entries(select(.value | test("is a separate task$")))
                        | map_values(1))}' \
        "$repo/.config/jscpd-baseline-reasons.json" >"$debt"
    # The gate's own flags, so the arm and the gate agree on what a clone is.
    # Its baseline is the arm's own: the gate's marks every tolerated clone,
    # and the reasons file already says which of them are debt.
    flags="$(just --justfile .just/lint.just --evaluate DRY_FLAGS \
        | sed 's/--baseline [^ ]*//')"
    case "$flags" in
        *--baseline*)
            rm -rf "$out"
            echo "ask-refactoring: the gate's flags carry a baseline the arm cannot replace" >&2
            exit 1
            ;;
    esac
    set +e
    # shellcheck disable=SC2086
    jscpd $flags --baseline "$debt" -r json -o "$out" crates >/dev/null 2>&1
    rc=$?
    set -e
    if ! jq -e '.duplicates | type == "array"' "$out/jscpd-report.json" >/dev/null 2>&1; then
        rm -rf "$out"
        echo "ask-refactoring: jscpd failed (exit $rc) or wrote no json report" >&2
        exit 1
    fi
    # Every debt fingerprint matches one pair. A report that tolerates fewer
    # pairs than the reasons file records means the two disagree about the
    # clones. The arm refuses rather than file a subset without saying so.
    local want got
    want="$(jq '.fingerprints | length' "$debt")"
    got="$(jq '[.duplicates[] | select(.isNew | not)] | length' "$out/jscpd-report.json")"
    if [ "$got" -lt "$want" ]; then
        rm -rf "$out"
        echo "ask-refactoring: $want debt fingerprints, $got pairs in the report; the reasons file and jscpd disagree" >&2
        exit 1
    fi
    score_dry <"$out/jscpd-report.json"
    rm -rf "$out"
}

emit_dry() {
    local path="$1" line="$2" copies="$3" tokens="$4" gain="$5" files="$6"
    local fragment="$7" locations="$8" impact="$9" ease="${10}"

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
  - and $more more, which \`just advisory ask-refactoring\` lists again on its next run"

    local measured="+ 🔴 This clone appears $copies times across $files $([ "$files" -eq 1 ] && echo file || echo files), $tokens jscpd tokens at most. Extracting it saves about $gain tokens net of the helper and its calls.
$listed"

    local remedy="- \`$path\` and every other file the AS-IS lists: extract a helper and call it from each copy, or mark the copies as intended with \`// jscpd:ignore-start\` and \`// jscpd:ignore-end\` and a comment saying why the match is a coincidence. Either way, re-mint the baseline with \`just lint::dry-baseline\` in the same commit."

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
        --set "goals=$(goals_with_one_helps maintainer keep-changes-safe 'a clone the gate tolerates lives in one helper, or carries a comment saying why it matches, and the baseline shrinks')"
        --set 'tags+=swept' --set 'tags+=dry' --set 'tags+=stateless'
    )
    local advice
    advice="$(printf '%s\n' \
        'the stem is taken, so this clone was filed before and closed' \
        'without the baseline losing it — reopen it, or mark the copies with' \
        '`jscpd:ignore-start` and a comment saying why, then `just lint::dry-baseline`')"
    file_finding dry "$stem" "[$copies copies, gain $gain]" "$advice" "${args[@]}"
}
