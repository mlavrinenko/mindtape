#!/usr/bin/env bash
# .just/scripts/_jq-reports-lib.sh: the readers behind `just meta::jq-reports`,
# sourced by check-jq-reports.sh and by the control that pins them
# (_jq-reports-control.sh). check-jq-reports.sh argues what each reads.
#
# The report program owns the answer to "what does it leave out". A report
# prints the config's top-level keys it covers and declares the rest in a
# `dropped` array. The gate reads that array with `jq -n -f`, which gives the
# program a null input. Neither the gate nor the control needs a live config
# to learn it. These functions compare that answer with the committed
# config-show schema, the one Rust-generated list of every top-level key.

# `config_keys <schema>`: one top-level config key per line.
config_keys() {
    jq -r '."$defs".Config.properties | keys[]' "$1"
}

# `printed_keys <report-output-json>`: the top-level keys a report printed.
printed_keys() {
    jq -r 'keys[]' "$1"
}

# `dropped_keys <report.jq>`: the report's own `dropped` array, one key per
# line. A report ends with `if . == null then dropped else ... end`. `-n` gives
# it a null input and prints the array. A real config prints the report.
dropped_keys() {
    jq -n -f "$1" | jq -r 'if type == "array" then .[] else error("dropped is not an array") end'
}

# `report_diff <report> <fixture> <expected> <actual> <err>` runs one report
# against its fixture, then diffs the output with the expected file. A diff
# failure still writes the output to `<actual>`, and the caller can compare
# keys anyway. jq's stderr lands in `<err>` for the failure message.
report_diff() {
    local report=$1 fixture=$2 expected=$3 actual=$4 err=$5
    if ! jq -f "$report" "$fixture" >"$actual" 2>"$err"; then
        echo "check-jq-reports: jq -f $report $fixture failed" >&2
        sed 's/^/  /' "$err" >&2
        return 1
    fi
    if ! diff -u "$expected" "$actual" >&2; then
        echo "check-jq-reports: $report does not match $expected" >&2
        return 1
    fi
}

# `report_key_drift <report> <schema> <printed-json>` compares what the report
# prints and drops with the schema's top-level keys. Each of four directions
# fails. The report skips a key, prints a key the config does not carry, drops
# one it does not carry, or does both to one key. The first is the check the
# gate exists for.
report_key_drift() {
    local report=$1 schema=$2 printed=$3 dropped known missing extra unknown overlap status=0
    dropped=$(mktemp) || return 1
    if ! dropped_keys "$report" >"$dropped" 2>/dev/null; then
        echo "check-jq-reports: $report did not print a \`dropped\` array for a null input" >&2
        echo "  end the program with \`if . == null then dropped else ... end\`" >&2
        rm -f "$dropped"
        return 1
    fi
    known=$(mktemp) || {
        rm -f "$dropped"
        return 1
    }
    config_keys "$schema" | sort -u >"$known"
    missing=$(comm -23 "$known" <( { printed_keys "$printed"; cat "$dropped"; } | sort -u))
    extra=$(comm -13 "$known" <(printed_keys "$printed" | sort -u))
    unknown=$(comm -13 "$known" <(sort -u "$dropped"))
    overlap=$(comm -12 <(printed_keys "$printed" | sort -u) <(sort -u "$dropped"))
    if [ -n "$missing" ]; then
        printf 'check-jq-reports: %s neither prints nor drops a top-level config key:\n' "$report" >&2
        # shellcheck disable=SC2001 # ${var//} replaces one match, and this prefixes every line
        sed 's/^/  /' <<<"$missing" >&2
        echo "  print it, or name it in the report's \`def dropped: [...]\`" >&2
        status=1
    fi
    if [ -n "$extra" ]; then
        printf 'check-jq-reports: %s prints a key the config schema does not carry:\n' "$report" >&2
        # shellcheck disable=SC2001 # ${var//} replaces one match, and this prefixes every line
        sed 's/^/  /' <<<"$extra" >&2
        status=1
    fi
    if [ -n "$unknown" ]; then
        printf 'check-jq-reports: %s drops a key the config schema does not carry:\n' "$report" >&2
        # shellcheck disable=SC2001 # ${var//} replaces one match, and this prefixes every line
        sed 's/^/  /' <<<"$unknown" >&2
        status=1
    fi
    if [ -n "$overlap" ]; then
        printf 'check-jq-reports: %s both prints and drops:\n' "$report" >&2
        # shellcheck disable=SC2001 # ${var//} replaces one match, and this prefixes every line
        sed 's/^/  /' <<<"$overlap" >&2
        status=1
    fi
    rm -f "$dropped" "$known"
    return "$status"
}
