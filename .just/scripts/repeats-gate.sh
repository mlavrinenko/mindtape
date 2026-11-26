#!/usr/bin/env bash
# .just/scripts/repeats-gate.sh: the body of the `lint::repeats` and
# `lint::repeats-baseline` recipes. Runs from the repo root, like the recipes
# that invoke it.
#
# The gate fails on a short repeat: a snippet of 15 jscpd tokens or more that
# the tree already holds, pasted again. `lint::dry` owns the clones at 50
# tokens and up, renamed or similar. This gate owns the exact tokens below
# that, where a pasted setup line lives.
#
# The report is jscpd's own. jscpd 5.4.3 has no `classes` array, and the
# `duplicates` pairs are the native source. Each pair carries a fragment, its
# token count, its two copies and whether the baseline holds it. The gate
# groups the pairs by fragment text and counts the unique copies by
# `name:start:end`. It scores each group by the tokens an extraction removes,
# net of the helper and its calls:
#
#   gain = (copies - 1) * tokens - 5 * copies - 10
#
# `tokens` is the smallest count jscpd measured for the fragment. A group
# clears the bar only when every measured copy is that big. The noise filter
# drops a fragment of nothing but `use`, `mod`, attribute and bracket lines:
# the shape of a Rust file, not a pasted idea. So is one with under 30
# characters left once those lines go. The fold keeps the highest-gain
# group first and drops one whose copies (half or more) fall on lines a kept
# group already holds. One setup line then reports once, not once per window.
#
# The bar is 100. Measured on the tree after the test-helper dedupe, at 15
# tokens with the fold: 100 selected 65 groups, 200 selected 9, 300 selected
# none. A sample of 20 at 100 held 2 real repeats (10%), at 200 one of 9
# (11%). So 100 is the lowest bar that clears the 10% floor. A 3-line setup
# pasted ten times scores about 210 and clears it. The same pasted five times
# scores about 85 and waits for the dry gate's 50-token floor.
#
# The baseline is jscpd's native `--baseline`, one fingerprint per pair, in
# `.config/jscpd-repeats-baseline.json`. It records the jscpd version that
# minted it. The gate refuses a baseline another binary minted. 5.4.3 dropped
# 161 fingerprints that 5.4.1 found in the same tree, and a mismatch would
# report every pair as new. A selected group fails when any of its pairs is
# new. A group the baseline tolerates needs a reason in
# `.config/jscpd-repeats-reasons.json`, keyed by the first 16 hex digits of
# the fragment's SHA-1. The reasons are the audit trail for every repeat the
# gate lets through. `--mint` rewrites the baseline, stamps the running
# version into it, and lists each selected group that still lacks a reason. A
# new repeat then cannot enter the baseline silently.
#
# The reasons are per group, not per pair. The baseline holds thousands of
# pair fingerprints, and a reason per pair would be a file nobody reads. The
# group is what the gate reports and what a maintainer decides about.
set -eo pipefail

BAR=100
TOKENS=15
BASELINE=.config/jscpd-repeats-baseline.json
REASONS=.config/jscpd-repeats-reasons.json
FLAGS=(--no-tips -f rust -k "$TOKENS" -l 1)

mint=0
case "${1:-}" in
    --mint) mint=1 ;;
    "") ;;
    *)
        echo "repeats-gate: unknown argument $1" >&2
        exit 2
        ;;
esac

if [ "$mint" -eq 0 ] && [ ! -f "$BASELINE" ]; then
    echo "repeats-gate: $BASELINE is missing; run \`just lint repeats-baseline\`" >&2
    exit 1
fi
if [ ! -f "$REASONS" ]; then
    echo "repeats-gate: $REASONS is missing; add it with one reason per tolerated group" >&2
    exit 1
fi

# The gate reads the baseline the running jscpd minted. Another version stops
# it before the detector runs, and the message states both versions. `--mint`
# adopts the running version instead.
if [ "$mint" -eq 0 ]; then
    minted=$(jq -r '.jscpd // empty' "$BASELINE")
    if [ -z "$minted" ]; then
        echo "error: $BASELINE names no jscpd version" >&2
        echo "hint: re-mint it: \`just lint repeats-baseline\`" >&2
        exit 1
    fi
    running=$(jscpd --version | awk '{print $NF}')
    if [ "$running" != "$minted" ]; then
        echo "error: jscpd $running is not the $minted that minted $BASELINE" >&2
        echo "hint: re-enter the dev shell: \`direnv reload\`" >&2
        exit 1
    fi
fi

report=$(mktemp -d)
trap 'rm -rf "$report"' EXIT

# The gate reads the report, not jscpd's exit status. A non-zero status says
# only that jscpd found a duplicate.
if [ "$mint" -eq 1 ]; then
    jscpd "${FLAGS[@]}" --baseline "$BASELINE" --update-baseline \
        -r json -o "$report" crates >/dev/null 2>&1 || true
    # jscpd rewrites the baseline with `version` and `fingerprints` alone.
    # The minting version goes back in after it writes.
    if [ -f "$BASELINE" ]; then
        version=$(jscpd --version | awk '{print $NF}')
        jq --arg v "$version" '{version, jscpd: $v, fingerprints}' "$BASELINE" >"$BASELINE.tmp"
        mv "$BASELINE.tmp" "$BASELINE"
    fi
else
    jscpd "${FLAGS[@]}" --baseline "$BASELINE" \
        -r json -o "$report" crates >/dev/null 2>&1 || true
fi

if ! jq -e '.duplicates | type == "array"' "$report/jscpd-report.json" >/dev/null 2>&1; then
    echo "repeats-gate: jscpd wrote no json report" >&2
    exit 1
fi

# Group, filter, score, fold, and keep the groups at the bar.
cat >"$report/select.jq" <<'JQ'
def noise: test("^\\s*(#!?\\[.*\\]|(pub(\\([a-z]+\\))? )?use .*;|(pub )?mod .*;|[{}()\\];,]*)\\s*$");
def lines_of($f): [range($f.start; $f.end + 1)] | map("\($f.name):\(.)");
def key($f): "\($f.name):\($f.start):\($f.end)";

[.duplicates[]
 | {fragment, tokens,
    body: ([.fragment | split("\n")[] | select(noise | not)] | join(" ") | gsub("\\s+"; " ")),
    new: .isNew,
    frags: ([.firstFile, .secondFile] | map({name, start, end}))}]
| group_by(.fragment)
| map({fragment: .[0].fragment,
       tokens: ([.[].tokens] | min),
       body: .[0].body,
       new: ([.[].new] | any),
       new_frags: ([.[] | select(.new) | .frags[]] | unique_by(key(.))),
       frags: ([.[].frags[]] | unique_by(key(.)))})
| map(select((.body | length) >= 30))
| map(. + {copies: (.frags | length)})
| map(. + {gain: ((.copies - 1) * .tokens - 5 * .copies - 10)})
| map(select(.gain >= $bar))
| sort_by(-.gain, .frags[0].name, .frags[0].start)
| reduce .[] as $g ({kept: [], claimed: {}};
    . as $st
    | ([$g.frags[] | select(any(lines_of(.)[]; $st.claimed[.] == true))] | length) as $hit
    | if $hit * 2 >= $g.copies then .
      else .kept += [$g] | .claimed += ([$g.frags[] | lines_of(.)[] | {(.): true}] | add)
      end)
| .kept
JQ

jq --argjson bar "$BAR" -f "$report/select.jq" "$report/jscpd-report.json" >"$report/classes.json"

# One JSON object per selected group, one line each.
jq -c '.[]' "$report/classes.json" >"$report/classes.jsonl"

fail=0
while IFS= read -r class; do
    fragment=$(jq -r '.fragment' <<<"$class")
    hash=$(printf '%s' "$fragment" | sha1sum | cut -c1-16)
    gain=$(jq -r '.gain' <<<"$class")
    copies=$(jq -r '.copies' <<<"$class")
    tokens=$(jq -r '.tokens' <<<"$class")
    if [ "$mint" -eq 0 ] && [ "$(jq -r '.new' <<<"$class")" = true ]; then
        printf 'repeats: a new %s-token repeat, gain %s, %s copies\n' "$tokens" "$gain" "$copies" >&2
        jq -r '.new_frags[] | "  crates/\(.name):\(.start)"' <<<"$class" >&2
        printf '%s\n' "$fragment" | sed 's/^/  | /' >&2
        fail=1
    elif ! jq -e --arg h "$hash" 'has($h)' "$REASONS" >/dev/null; then
        printf 'repeats: no reason for a tolerated %s-token repeat, gain %s, %s copies at %s\n' \
            "$tokens" "$gain" "$copies" "$(jq -r '.frags[0] | "crates/\(.name):\(.start)"' <<<"$class")" >&2
        printf '  add a reason under "%s" in %s\n' "$hash" "$REASONS" >&2
        printf '%s\n' "$fragment" | sed 's/^/  | /' >&2
        fail=1
    fi
done <"$report/classes.jsonl"

if [ "$fail" -ne 0 ]; then
    if [ "$mint" -eq 1 ]; then
        echo "repeats-gate: the baseline is re-minted; add the reasons above and re-run" >&2
    else
        echo "repeats-gate: extract a helper, or add a reason for a coincidence" >&2
    fi
    exit 1
fi

if [ "$mint" -eq 1 ]; then
    echo "repeats-gate: baseline re-minted, every tolerated group has a reason"
else
    echo "repeats: no new short repeat"
fi
