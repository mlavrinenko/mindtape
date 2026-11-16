#!/usr/bin/env bash
# .just/scripts/check-vale-levels.sh: the body of the `lint::vale-levels`
# recipe (truth/tasks/2026-11/forbid-warning-levels-in-vale-configs.typ). Runs
# from the repo root, like the recipe that invokes it.
#
# Vale has three levels, `error`, `warning` and `suggestion`. A rule below
# `error` shows in an editor and passes every gate, so nobody has to fix it.
# This gate reads every `.config/vale*.ini` and requires:
#
#   - `MinAlertLevel = error`, so a config cannot default to warnings.
#   - every rule line at `error` or `NO`.
#   - every rule of every style a section enables, because a new rule file in
#     `target/vale/styles` otherwise inherits a quiet level.
#
# The style rule names come from `target/vale/styles/*/*.yml`, which the dev
# shell links from the pinned nix build. A style with no directory (`Vale`, the
# tool's built-in style) has no rule file to read, so this gate checks only the
# config's own lines for it. `vale ls-config` cannot stand in for this file: it
# prints each rule as a boolean, so `warning` and `error` both read `true`, and
# it omits a rule the config does not name.
#
# A stale style is a silent gate change. The dev shell links
# `target/vale/styles` at its nix build, and a cached direnv or nix evaluation
# can keep serving the old one after an edit. The drift check catches a stale
# link. Every overlay file must match its built copy, and each verb dictionary
# must be older than the `Imperative.tengo` the build embeds it into.
#
# The control runs first over a synthetic style and configs. A parser that
# stopped reading sections would report a clean tree, which is exactly what a
# correct gate reports. The fixture pins the findings and their order.
set -eo pipefail

styles=target/vale/styles
shopt -s nullglob

# Print `Style<TAB>Rule` for every rule file under a styles directory.
rule_list() {
    local dir=$1 f rel style rule
    for f in "$dir"/*/*.yml; do
        rel=${f#"$dir"/}
        style=${rel%%/*}
        rule=${rel#*/}
        rule=${rule%.yml}
        printf '%s\t%s\n' "$style" "$rule"
    done
}

# Print each finding in <config> against the rules in <rules>.
check() {
    local config=$1 rules=$2
    awk -v config="$config" '
        FNR == NR {
            style = $1
            rule = $2
            rule_count[style]++
            rule_order[style, rule_count[style]] = rule
            next
        }
        {
            line = $0
            sub(/^[ \t]+/, "", line)
            sub(/[ \t]+$/, "", line)
            if (line == "" || line ~ /^[#;]/) next
            if (line ~ /^\[.*\]$/) {
                section = line
                sub(/^\[/, "", section)
                sub(/\]$/, "", section)
                gsub(/^[ \t]+|[ \t]+$/, "", section)
                if (!(section in seen_section)) {
                    seen_section[section] = 1
                    sections[++section_count] = section
                }
                next
            }
            eq = index(line, "=")
            if (eq == 0) next
            key = substr(line, 1, eq - 1)
            val = substr(line, eq + 1)
            gsub(/^[ \t]+|[ \t]+$/, "", key)
            gsub(/^[ \t]+|[ \t]+$/, "", val)
            sub(/[ \t]+[#;].*$/, "", val)
            gsub(/^[ \t]+|[ \t]+$/, "", val)
            value[section, key] = val
            low = tolower(val)
            if (key == "MinAlertLevel") {
                seen_min_alert = 1
                if (low != "error") {
                    printf "check-vale-levels: %s:%d: MinAlertLevel = %s; set it to error\n",
                        config, FNR, val
                    failed = 1
                }
            } else if (key ~ /\./) {
                if (low == "warning") {
                    printf "check-vale-levels: %s:%d: %s = warning; a warning is a rule no gate enforces\n",
                        config, FNR, key
                    failed = 1
                } else if (low != "error" && low != "no") {
                    printf "check-vale-levels: %s:%d: %s = %s; set it to error or NO\n",
                        config, FNR, key, val
                    failed = 1
                }
            }
        }
        END {
            if (!seen_min_alert) {
                printf "check-vale-levels: %s: no MinAlertLevel = error; the default level is warning\n",
                    config
                failed = 1
            }
            for (i = 1; i <= section_count; i++) {
                section = sections[i]
                if (!((section, "BasedOnStyles") in value)) continue
                n = split(value[section, "BasedOnStyles"], names, /[ \t]*,[ \t]*/)
                for (j = 1; j <= n; j++) {
                    style = names[j]
                    for (k = 1; k <= rule_count[style]; k++) {
                        rule = rule_order[style, k]
                        key = style "." rule
                        if (!((section, key) in value)) {
                            printf "check-vale-levels: %s: [%s] enables %s but gives %s neither error nor NO\n",
                                config, section, style, key
                            failed = 1
                        }
                    }
                }
            }
            exit failed
        }
    ' "$rules" "$config"
}

# Print a finding for every overlay file or dictionary the built <style>
# copy left behind. The build copies the overlay verbatim. A differing copy is
# a stale link. It embeds the dictionaries into the verb list of
# `Imperative.tengo`, and the check reads that list back to compare it with
# the dictionaries.
drift() {
    local overlay=$1 style=$2 dicts=$3 f status=0
    while IFS= read -r f; do
        if ! cmp -s "$overlay/$f" "$style/$f"; then
            printf 'check-vale-levels: %s/%s differs from the built copy; run direnv reload\n' \
                "$overlay" "$f"
            status=1
        fi
    done < <(cd "$overlay" && find . -type f | sed 's|^\./||' | sort)
    if ! diff -q \
        <(awk '/^verbs := \[/ { inlist = 1; next }
            /^\]/ { inlist = 0 }
            inlist { gsub(/[",[[:space:]]/, ""); print }' \
            "$style/config/scripts/Imperative.tengo") \
        <(awk '!/^#/ && NF { print $1 }' "$dicts/verbs.txt" "$dicts/dev-verbs.txt") \
        >/dev/null; then
        printf 'check-vale-levels: %s disagrees with the verbs in the built Imperative.tengo; run direnv reload\n' \
            "$dicts"
        status=1
    fi
    return "$status"
}

# The fixture. It runs against its own style and configs, never the built style,
# so it proves the parser and the completeness walk without depending on a
# flake update's rule set.
control() {
    local dir=$1 good=$1/good.ini bad=$1/bad.ini missing=$1/missing.ini rules=$1/rules
    local got status want
    mkdir -p "$dir/styles/Prose"
    : >"$dir/styles/Prose/Kept.yml"
    : >"$dir/styles/Prose/Dropped.yml"
    rule_list "$dir/styles" >"$rules"

    cat >"$good" <<'EOF'
MinAlertLevel = error
[*.md]
BasedOnStyles = Prose, Vale
Prose.Kept = error
Prose.Dropped = NO
Vale.Terms = error
EOF
    if ! got=$(check "$good" "$rules" 2>&1); then
        printf 'check-vale-levels: the control good config failed:\n%s\n' "$got" >&2
        exit 1
    fi
    if [ -n "$got" ]; then
        printf 'check-vale-levels: the control good config reported:\n%s\n' "$got" >&2
        exit 1
    fi

    cat >"$bad" <<'EOF'
MinAlertLevel = warning
[*.md]
BasedOnStyles = Prose
Prose.Kept = warning
[*.rst]
BasedOnStyles = Prose
Prose.Kept = error
Prose.Dropped = suggestion
EOF
    want=$(cat <<EOF
check-vale-levels: $bad:1: MinAlertLevel = warning; set it to error
check-vale-levels: $bad:4: Prose.Kept = warning; a warning is a rule no gate enforces
check-vale-levels: $bad:8: Prose.Dropped = suggestion; set it to error or NO
check-vale-levels: $bad: [*.md] enables Prose but gives Prose.Dropped neither error nor NO
EOF
    )
    status=0
    got=$(check "$bad" "$rules" 2>&1) || status=$?
    if [ "$status" -eq 0 ] || [ "$got" != "$want" ]; then
        printf 'check-vale-levels: the control bad config gave\n%s\nexpected\n%s\n' \
            "$got" "$want" >&2
        exit 1
    fi

    cat >"$missing" <<'EOF'
[*.md]
BasedOnStyles = Prose
Prose.Kept = error
Prose.Dropped = NO
EOF
    want="check-vale-levels: $missing: no MinAlertLevel = error; the default level is warning"
    status=0
    got=$(check "$missing" "$rules" 2>&1) || status=$?
    if [ "$status" -eq 0 ] || [ "$got" != "$want" ]; then
        printf 'check-vale-levels: the control missing-level config gave\n%s\nexpected\n%s\n' \
            "$got" "$want" >&2
        exit 1
    fi

    # The drift fixture. A matching file passes, a differing file and a
    # disagreeing verb list each name one finding.
    local overlay=$1/drift-overlay style=$1/drift-style dicts=$1/drift-dicts
    mkdir -p "$overlay/Prose" "$overlay/Commits" "$style/Prose" \
        "$style/Commits" "$style/config/scripts" "$dicts"
    echo one >"$overlay/Prose/words.txt"
    echo one >"$style/Prose/words.txt"
    echo old >"$overlay/Commits/History.yml"
    echo new >"$style/Commits/History.yml"
    printf 'verbs := [\n  "run",\n  "fix"\n]\n' >"$style/config/scripts/Imperative.tengo"
    printf '# a comment\nrun\nfix\n' >"$dicts/verbs.txt"
    : >"$dicts/dev-verbs.txt"
    want=$(cat <<EOF
check-vale-levels: $overlay/Commits/History.yml differs from the built copy; run direnv reload
EOF
    )
    status=0
    got=$(drift "$overlay" "$style" "$dicts" 2>&1) || status=$?
    if [ "$status" -eq 0 ] || [ "$got" != "$want" ]; then
        printf 'check-vale-levels: the control drift fixture gave\n%s\nexpected\n%s\n' \
            "$got" "$want" >&2
        exit 1
    fi

    printf 'verbs := [\n  "run"\n]\n' >"$style/config/scripts/Imperative.tengo"
    want=$(cat <<EOF
check-vale-levels: $overlay/Commits/History.yml differs from the built copy; run direnv reload
check-vale-levels: $dicts disagrees with the verbs in the built Imperative.tengo; run direnv reload
EOF
    )
    status=0
    got=$(drift "$overlay" "$style" "$dicts" 2>&1) || status=$?
    if [ "$status" -eq 0 ] || [ "$got" != "$want" ]; then
        printf 'check-vale-levels: the control stale-verbs fixture gave\n%s\nexpected\n%s\n' \
            "$got" "$want" >&2
        exit 1
    fi
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
control "$tmp/control"

rules=$tmp/rules
rule_list "$styles" >"$rules"
if [ ! -s "$rules" ]; then
    echo "check-vale-levels: no rule files under $styles — enter the dev shell so the style builds" >&2
    exit 1
fi

configs=(.config/vale*.ini)
if [ ${#configs[@]} -eq 0 ]; then
    echo "check-vale-levels: no .config/vale*.ini found — running outside the repo root?" >&2
    exit 1
fi

fail=0
if ! drift .config/vale/local "$(readlink -f "$styles")" crates/mindtape/src/dicts; then
    echo "check-vale-levels: a stale style reads old rules and old words" >&2
    fail=1
fi
for config in "${configs[@]}"; do
    check "$config" "$rules" || fail=1
done

if [ "$fail" -ne 0 ]; then
    echo "check-vale-levels: a rule below error is a rule no gate enforces" >&2
fi
exit "$fail"
