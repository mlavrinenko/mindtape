#!/usr/bin/env bash
# .just/scripts/_jq-reports-control.sh: the control `just meta::jq-reports`
# runs before it reads the real reports. check-jq-reports.sh says why a control
# exists. Runs from the repo root. Builds synthetic reports in a temp dir, never
# the working tree: `just check` runs its arms in parallel, so a probe there
# would fail a concurrent gate.
#
# A fixture comparison fails quietly in two ways. A `dropped` array that
# nothing reads, and a key comparison that matches everything. Either prints
# the same "no findings" line as a correct run. So the clean report must pass,
# and each bad one must fail with the offending key named.
set -eo pipefail

# shellcheck source=.just/scripts/_jq-reports-lib.sh
source .just/scripts/_jq-reports-lib.sh

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail=0

problem() {
    echo "check-jq-reports: control: $*" >&2
    fail=1
}

cat >"$tmp/schema.json" <<'EOF'
{
  "$defs": {
    "Config": {
      "properties": {
        "a": {},
        "b": {},
        "c": {}
      }
    }
  }
}
EOF

# A report is a jq program with a `dropped` definition and the null-input
# dispatch the gate reads. Each case below is this program with one answer
# changed.
write_report() { # <file> <dropped-array> <printed-object>
    printf 'def dropped: %s;\nif . == null then dropped else %s end\n' "$2" "$3" >"$1"
    jq -f "$1" <<<'{}' >"$1.json"
}

# `expect_drift <label> <report> <needle>`: the report must fail key drift, and
# the failure must name `<needle>`.
expect_drift() {
    local label=$1 report=$2 needle=$3 out
    if out=$(report_key_drift "$report" "$tmp/schema.json" "$report.json" 2>&1); then
        problem "$label: expected a failure, got 0"
    elif ! grep -qF -- "$needle" <<<"$out"; then
        problem "$label: failed but never named $needle"
        printf '%s\n' "$out" >&2
    fi
}

# A clean report prints `a` and drops `b` and `c`. It must pass.
write_report "$tmp/clean.jq" '["b", "c"]' '{a: 1}'
out=$(report_key_drift "$tmp/clean.jq" "$tmp/schema.json" "$tmp/clean.jq.json" 2>&1) ||
    problem "clean report failed: $out"

# A key neither printed nor dropped. This is the arm the gate exists for.
write_report "$tmp/missing.jq" '["b"]' '{a: 1}'
expect_drift "missing key" "$tmp/missing.jq" "c"

# A dropped key the schema does not carry: the array rotted.
write_report "$tmp/unknown.jq" '["b", "c", "gone"]' '{a: 1}'
expect_drift "unknown dropped key" "$tmp/unknown.jq" "gone"

# A key both printed and dropped, which says two things at once.
write_report "$tmp/overlap.jq" '["a", "b", "c"]' '{a: 1}'
expect_drift "printed and dropped" "$tmp/overlap.jq" "a"

# A printed key the config does not carry.
write_report "$tmp/extra.jq" '["b", "c"]' '{a: 1, gone: 2}'
expect_drift "extra printed key" "$tmp/extra.jq" "gone"

# A program with no `dropped` dispatch prints its report for a null input.
printf '{a: 1}\n' >"$tmp/no-dropped.jq"
jq -f "$tmp/no-dropped.jq" <<<'{}' >"$tmp/no-dropped.jq.json"
expect_drift "no dropped array" "$tmp/no-dropped.jq" "dropped"

# The diff arm catches a report whose content drifted.
printf '{}\n' >"$tmp/fixture.json"
printf '{\n  "a": 1\n}\n' >"$tmp/expected.json"
if ! report_diff "$tmp/clean.jq" "$tmp/fixture.json" "$tmp/expected.json" "$tmp/actual.json" "$tmp/err" 2>"$tmp/diff.err"; then
    problem "report_diff positive control failed: $(cat "$tmp/diff.err")"
fi
printf '{\n  "a": 2\n}\n' >"$tmp/expected.json"
if report_diff "$tmp/clean.jq" "$tmp/fixture.json" "$tmp/expected.json" "$tmp/actual.json" "$tmp/err" 2>"$tmp/diff.err"; then
    problem "report_diff accepted an output that does not match"
elif ! grep -qF "$tmp/expected.json" "$tmp/diff.err"; then
    problem "report_diff failed but never named the expected file"
fi

if [ "$fail" -ne 0 ]; then
    echo "check-jq-reports: the control report gate misses a case it claims to catch" >&2
    exit 1
fi
