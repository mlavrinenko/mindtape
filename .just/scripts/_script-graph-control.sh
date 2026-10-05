#!/usr/bin/env bash
# .just/scripts/_script-graph-control.sh — the control fixture for
# .just/scripts/_script-graph.sh, run by script-scope-check.sh immediately
# before its arm two (truth/tasks/follow-script-to-script-calls-in-the-scope-
# derivation.typ).
#
# It exists because the thing it guards fails green, in both directions, and
# they are opposite failures with the same printout. A derivation that quietly
# stopped following an edge reports every rule covered — which is what a correct
# manifest reports — and puts back exactly the silent stale hit the transitive
# walk was written to remove. A derivation that reads one word too many demands
# a scope for a body no rule runs, and that direction is worse: it fails a
# correct manifest, and the obvious repair is to add the input rather than to
# fix the reader, so the wrong edge becomes permanent the first time someone is
# in a hurry. Neither can be seen from the real tree, where the answer and the
# bug are the same sentence.
#
# A separate file rather than a `--control` arm inside the walk, because a
# self-test is its own subject, and the
# walk it drives is already the longest reasoning in that directory. No `just`
# recipe of its own either — it asserts nothing a reader would run on purpose,
# and `just script-scope-check` running it first is what makes arm two
# trustworthy. This costs no hand-wiring: the walk
# reads script-scope-check.sh naming both files and derives them, which is the
# feature demonstrating itself.
#
# A synthetic tree, never the real one. The shapes below have to sit next to
# each other in one pass — a real script carries one or two of them — and three
# of the five cannot exist in the tracked tree at all, because two of them are
# gate failures and the third is a path to a file that does not exist.
#
# Five cases on three runs, four loud and one silent, because each can rot
# without touching another: the script-to-script hop, the nested-`just` hop, the
# unclassified flag, the unreadable target, and — the silent one — every
# not-a-call shape staying out of the graph. The last is not decoration. It is
# the property that keeps a script that merely names another out of the graph.
set -eo pipefail

graph=".just/scripts/_script-graph.sh"
if [ ! -f "$graph" ]; then
    echo "script-graph-control: $graph is missing — running outside the repo root?" >&2
    exit 1
fi
graph=$PWD/$graph

ctl=$(mktemp -d)
trap 'rm -rf "$ctl"' EXIT
mkdir -p "$ctl/.just/scripts"

# `alpha` runs a.sh, which reaches b.sh script-to-script and c.sh through a
# nested `just`. Nothing here names b.sh or c.sh from a recipe body, so a walk
# that follows only direct invocation finds neither.
cat >"$ctl/dump.json" <<'EOF'
{
  "recipes": {
    "alpha":  {"namepath": "alpha",  "body": [["bash .just/scripts/a.sh"]]},
    "nested": {"namepath": "nested", "body": [["bash .just/scripts/c.sh"]]},
    "lonely": {"namepath": "lonely", "body": [["bash .just/scripts/lonely.sh"]]}
  }
}
EOF

cat >"$ctl/.just/scripts/a.sh" <<'EOF'
bash .just/scripts/b.sh
reason=$(just nested)
EOF

# Every not-a-call shape at once. The `just lonely` mentions are the ones that
# matter most: a hint in an error message is how every script in the real tree
# spells a recipe name it does not run, and reading one as an edge would demand
# `script-lonely` on a rule that never touches it.
cat >"$ctl/.just/scripts/b.sh" <<'EOF'
# bash .just/scripts/commented.sh
echo "run \`just lonely\` first, then bash .just/scripts/commented.sh" >&2
rule="just lonely"
probe=".just/scripts/absent.sh"
just --dump --dump-format json >/dev/null
just --evaluate LONELY_VAR >/dev/null
EOF

: >"$ctl/.just/scripts/c.sh"
: >"$ctl/.just/scripts/lonely.sh"
: >"$ctl/.just/scripts/commented.sh"

# `bash "$graph"`: nothing under .just/scripts/ carries an exec bit.
if ! out=$(cd "$ctl" && bash "$graph" dump.json 2>&1); then
    echo "script-graph-control: the walk FAILED on a tree it should read cleanly" >&2
    sed 's/^/  /' <<<"$out" >&2
    exit 1
fi

for want in \
    'alpha	.just/scripts/b.sh	just alpha -> .just/scripts/a.sh -> .just/scripts/b.sh' \
    'alpha	.just/scripts/c.sh	just alpha -> .just/scripts/a.sh -> just nested -> .just/scripts/c.sh'; do
    if ! grep -qF -- "$want" <<<"$out"; then
        echo "script-graph-control: the walk did not reach [${want//	/ | }]" >&2
        sed 's/^/  /' <<<"$out" >&2
        exit 1
    fi
done

# The silent case. commented.sh is named on a comment line and inside an error
# message, absent.sh is assigned to a variable and does not exist, and
# lonely.sh is reachable only by reading b.sh's `just lonely` hints as calls —
# so it is checked against alpha's rows rather than the whole output, `lonely`
# being a real recipe of its own that legitimately reaches it.
for unwanted in 'commented.sh' 'absent.sh'; do
    if grep -qF -- "$unwanted" <<<"$out"; then
        echo "script-graph-control: the walk read a non-call as an edge [$unwanted]" >&2
        sed 's/^/  /' <<<"$out" >&2
        exit 1
    fi
done
if grep '^alpha	' <<<"$out" | grep -qF -- 'lonely.sh'; then
    echo "script-graph-control: the walk read an error message's \`just lonely\` as a call" >&2
    sed 's/^/  /' <<<"$out" >&2
    exit 1
fi

# A flag nobody has classified, and a target no static read can resolve. Both
# must be loud: skipping either is how a real call stops being followed without
# anything saying so.
assert_refuses() {
    local case=$1 body=$2 want=$3 out
    printf '%s\n' "$body" >"$ctl/.just/scripts/a.sh"
    if out=$(cd "$ctl" && bash "$graph" dump.json 2>&1); then
        echo "script-graph-control: the walk ACCEPTED $case, so it would follow it silently" >&2
        sed 's/^/  /' <<<"$out" >&2
        exit 1
    fi
    if ! grep -qF -- "$want" <<<"$out"; then
        echo "script-graph-control: the walk refused $case without naming [$want]" >&2
        sed 's/^/  /' <<<"$out" >&2
        exit 1
    fi
}

assert_refuses "an unclassified flag" 'just --colour-me-surprised nested' \
    'carries a flag this walk has not been told about'
assert_refuses "an unreadable target" 'just "$recipe"' \
    'names no recipe in the dump'
