#!/usr/bin/env bash
# .just/scripts/_gitleaks-bound-check.sh — the walk-bound arm of `just
# check-secrets`, extracted out of .just/scripts/check-secrets.sh under
# truth/tasks/hold-the-root-gitignore-to-the-gitleaks-walk-bound.typ when
# holding the ROOT .gitignore to the same rule doubled its size against a
# 300-line cap that file was already at 96% of. Its subject was always the
# MANIFEST rather than the tree: nothing here scans for a secret, and the
# scan's verdict never depends on it.
#
# WHY THE BOUND EXISTS AT ALL, what the `(/|$)` form buys, and what no pattern
# can buy, are argued once beside the list in `.gitleaks.toml` and not
# restated here. What matters here is only that the list is a HAND COPY of two
# `.gitignore` files, and a hand copy drifts.
#
# Takes the gitleaks config as $1 and one or more `.gitignore` paths after it,
# deriving each file's prefix from its own location (`www/.gitignore` →
# `www/`), so a third gitignore is covered by being passed rather than by
# anything here learning about it.
#
# Leading underscore for _nav-diff.sh's reason: no `just` recipe wraps it,
# check-secrets.sh invokes it as a plain script. THAT COSTS THE SAME GATE —
# script-scope-check's arm two cannot see a script-to-script call, so
# `script-gitleaks-bound-check` is named BY HAND in `just check-secrets`'s
# inputs, with the reason recorded there.
#
# BOTH ARMS BELOW FAIL GREEN if this file stops working, so nothing here is
# trusted on its own: `.just/scripts/_gitleaks-bound-control.sh` drives them
# against a synthesized pair first and demands three named failures and one
# silence. check-secrets.sh runs it immediately before the real pair; its
# header argues why.
#
# TWO ARMS, the two directions one coupling can break:
#
#   1. every anchored .gitignore entry is covered by the bound, or carries an
#      exemption saying why not. The drift this closes: a planted
#      `aws-access-token` under gitignored `.direnv/` WAS reported, by a gate
#      whose whole claim is that it walks only the tracked tree.
#   2. no bound pattern matches a TRACKED file. An allowlist is the one thing
#      in a scanner that can only ever hide a finding, and the obvious way to
#      green arm one is to widen a pattern until it swallows something real.
#      `/.cargo/*` is exactly that shape — its tracked `!/.cargo/mutants.toml`
#      exception means `^\.cargo(/|$)` satisfies arm one while silencing a
#      committed file — so this arm makes that a red gate, not a review catch.
#
# THE EXEMPTION LIVES IN THE .gitignore, on the line above the entry, where
# the decision is legible next to what it decides about and a new entry
# inherits nothing. An entry with no marker above it is DEMANDED, so the entry
# nobody decided about is the entry that fails.
#
# The marker must OPEN its comment, and that is not tidiness: the paragraph in
# .gitignore documenting this convention quotes the marker mid-line, and a
# substring match read that prose as an exemption and silently exempted the
# entry under it (`/.direnv`, the very drift above). A directive any sentence
# mentioning it can cast is not a directive.
#
# THREE SHAPES ARE NOT DEMANDED, none of them an exemption.
#
# An UNANCHORED pattern (`.DS_Store`, `.goutputstream-*`) names a class of
# transient with no fixed location, and a path regex has nothing to say about
# it. Anchoring follows git's own rule rather than the leading slash alone: a
# pattern carrying a `/` anywhere but its end is anchored to its .gitignore's
# directory, which is the only reason `.mmz/cache/` is checked at all.
#
# A NEGATION (`!/.cargo/mutants.toml`) re-includes a tracked file — the
# opposite of scratch, and bounding it is what arm two refuses.
#
# And the BARE path is what must match, never a child: `^www/public/` skips
# every entry inside www/public/ while still descending into it, which bounds
# nothing. `^www/public(/|$)` is the form, measured in `.gitleaks.toml`.
#
# A `*` IN AN ENTRY IS TESTED THROUGH A WITNESS (`/skills/*-workspace` →
# `skills/GLOBWITNESS-workspace`), because the grep's subject is a path and a
# gitignore glob is not one. gitignore's `*` does not cross `/`, so a
# segment-safe stand-in is a fair probe: it passes a pattern mirroring the
# glob (`^skills/[^/]*-workspace(/|$)`) and fails one pinned to today's only
# instance. `?` and `[` have no such stand-in and are refused loudly.
set -eo pipefail

marker="gitleaks-bound: exempt"

config="$1"
shift
if [ ! -f "$config" ]; then
    echo "gitleaks-bound: $config is missing — running outside the repo root?" >&2
    exit 1
fi

# The allowlist is selected by carrying no `targetRules`: a rule-scoped one is
# a false-positive silencer, the unscoped one is the bound. tomlq is the `yq`
# package's TOML front end, the same dev-shell pin. `// []` on both reads: an
# allowlist that lost its `paths` key, or a config that lost `allowlists`, would
# otherwise make jq die on a null and `set -e` end this script with jq's own
# message instead of the sentence below — a config error reported as a tooling
# error, on the exact edit this guard exists to catch.
bound=$(tomlq -r '
    (.allowlists // [])[] | select(has("targetRules") | not) | (.paths // [])[]
' "$config")
if [ -z "$bound" ]; then
    echo "gitleaks-bound: $config declares no unscoped path allowlist" >&2
    echo "  the walk bound this check reads is gone — restore it, do not delete the check" >&2
    exit 1
fi

fail=0
checked=0
exempt=0
unbounded=""

for ignore in "$@"; do
    if [ ! -f "$ignore" ]; then
        echo "gitleaks-bound: $ignore does not exist" >&2
        exit 1
    fi
    # The prefix IS the argument minus its filename, so an argument that is
    # not named `.gitignore` yields a prefix that is the whole path and every
    # entry under it is reported as unbounded — a wrong answer dressed as a
    # finding. Refused rather than trusted, because the shape that produces it
    # is a caller pointing this check at a copy.
    if [ "${ignore%.gitignore}" = "$ignore" ]; then
        echo "gitleaks-bound: $ignore is not named \`.gitignore\`" >&2
        echo "  the path prefix each entry is checked under is derived from that name" >&2
        exit 1
    fi
    prefix=${ignore%.gitignore}
    marked=0
    reason=""
    while IFS= read -r line || [ -n "$line" ]; do
        # Only a comment carries a marker forward; every other line consumes
        # it. So a marker separated from its entry by a blank line exempts
        # nothing and the entry is demanded — loudly wrong rather than quietly
        # attached to whatever came next.
        if [ "${line#\#}" != "$line" ]; then
            body=${line#\#}
            body=${body# }
            case "$body" in
                "$marker"*)
                    marked=1
                    reason=$(tr -d ' —-' <<<"${body#"$marker"}")
                    ;;
            esac
            continue
        fi
        pending=$marked
        pending_reason="$reason"
        marked=0
        reason=""
        [ -n "$line" ] || continue
        [ "${line#!}" = "$line" ] || continue

        entry=${line%/}
        case "$entry" in
            /*) entry=${entry#/} ;;
            */*) ;;
            *) continue ;;
        esac

        # A bare marker with no reason after it is refused rather than
        # honoured: the exemption's whole value is the sentence saying why the
        # scan must still read this path, and an unexplained one is the shape
        # a drive-by "make the gate green" leaves behind.
        if [ "$pending" -eq 1 ]; then
            if [ -z "$pending_reason" ]; then
                echo "gitleaks-bound: $ignore exempts \`$entry\` with no reason" >&2
                echo "  write it after the marker: \`# $marker — <why bounding it would cost coverage>\`" >&2
                fail=1
                continue
            fi
            exempt=$((exempt + 1))
            continue
        fi

        path="$prefix$entry"
        case "$path" in
            *'?'* | *'['*)
                echo "gitleaks-bound: $ignore names \`$entry\`, a glob shape this check cannot probe" >&2
                echo "  teach the witness translation above about it, or exempt the entry" >&2
                fail=1
                continue
                ;;
        esac
        checked=$((checked + 1))
        grep -qE -f <(printf '%s\n' "$bound") <<<"${path//\*/GLOBWITNESS}" ||
            unbounded="$unbounded  $path  ($ignore)"$'\n'
    done <"$ignore"
done

if [ -n "$unbounded" ]; then
    echo "gitleaks-bound: gitignored paths the walk bound in $config misses" >&2
    printf '%s' "$unbounded" >&2
    echo "  each is scratch a parallel \`just check\` arm may be rewriting mid-scan," >&2
    echo "  so the walk must never enter it — add \`^<path>(/|\$)\` (a directory) or" >&2
    echo "  \`^<path>\$\` (a file) to the allowlist that declares no targetRules" >&2
    echo "  or, if bounding it would hide something the scan must read, put a" >&2
    echo "  \`# $marker — <why>\` line directly above the entry" >&2
    fail=1
fi

# ARM TWO. Tracked files only: an untracked one is what the bound exists to
# skip, so matching one is the intended outcome and matching a COMMITTED file
# is the failure. Symlinks are dropped (mode 120000) because gitleaks reads
# none without `--follow-symlinks` — measured, and the reason `/result`, a nix
# build output, is exempt from arm one rather than bounded by a pattern that
# would silence a real directory if the name were ever reused.
hidden=$(git ls-files -s | grep -v '^120000 ' | cut -f2- |
    grep -E -f <(printf '%s\n' "$bound") || true)
if [ -n "$hidden" ]; then
    n=$(wc -l <<<"$hidden")
    echo "gitleaks-bound: the walk bound in $config hides $n TRACKED file(s)" >&2
    head -5 <<<"$hidden" | while IFS= read -r path; do echo "  $path" >&2; done
    [ "$n" -gt 5 ] && echo "  … and $((n - 5)) more" >&2
    echo "  an allowlist can only ever hide a finding; a pattern wide enough to" >&2
    echo "  swallow a committed file removes the coverage this gate is for —" >&2
    echo "  narrow it, or exempt the .gitignore entry that asked for it" >&2
    fail=1
fi

[ "$fail" -eq 0 ] || exit 1
