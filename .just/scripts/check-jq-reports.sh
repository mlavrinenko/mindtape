#!/usr/bin/env bash
# .just/scripts/check-jq-reports.sh: the body of the `meta::jq-reports`
# recipe. Runs from the repo root, like the recipe that invokes it.
#
# This gate and the report it guards are temporary. Both retire when
# `git log --since="1 month ago" -- contrib/jq/reports/` is empty at a release
# review. That empty log means no one has had to correct the report for a
# month. Its shape and the `moments` table have then settled, and they can move
# into `mt`.
#
# Run every `contrib/jq/reports/*.jq` over its fixture and diff the output with
# the expected file beside the fixture. Then hold each report to the config's
# top-level keys. A report prints the keys it covers and names the rest in its
# own `dropped` array. The gate reads that array with `jq -n -f`. A key added to
# the config cannot fall through both lists.
#
# The gate reads the schema, not a live `mt config show`. `just check fast`
# runs this arm and has no built `target/debug/mt` to run. The committed
# `schema/output/config-show/` file is the one Rust-generated list of the
# config's top-level keys. `just test`'s `schema_e2e` holds that file to the
# types. A new key reaches here with the schema. The fixture is a recorded
# resolved config, which keeps the report diff deterministic and off the task
# corpus.
#
# The control runs first. check-secrets.sh gives the reason: "no findings" is
# equally what a broken reader prints. The readers live in
# `_jq-reports-lib.sh`. The body sources them, and so does the control.
set -eo pipefail

# shellcheck source=.just/scripts/_jq-reports-lib.sh
source .just/scripts/_jq-reports-lib.sh

bash .just/scripts/_jq-reports-control.sh

schema=schema/output/config-show/1.schema.json
reports_dir=contrib/jq/reports
fixtures_dir=contrib/jq/fixtures
fixture=$fixtures_dir/config.resolved.json

for required in "$schema" "$fixture"; do
    [ -f "$required" ] || {
        echo "check-jq-reports: $required is missing — running outside the repo root?" >&2
        exit 1
    }
done

shopt -s nullglob
reports=("$reports_dir"/*.jq)
if [ ${#reports[@]} -eq 0 ]; then
    echo "check-jq-reports: no reports under $reports_dir" >&2
    exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
status=0

for report in "${reports[@]}"; do
    stem=$(basename "$report" .jq)
    expected=$fixtures_dir/$stem.out.json
    if [ ! -f "$expected" ]; then
        echo "check-jq-reports: $report has no expected output at $expected" >&2
        echo "  add one with: jq -f $report $fixture > $expected" >&2
        status=1
        continue
    fi
    actual=$tmp/$stem.json
    report_diff "$report" "$fixture" "$expected" "$actual" "$tmp/$stem.err" || status=1
    [ -s "$actual" ] || continue
    report_key_drift "$report" "$schema" "$actual" || status=1
done

# A stale expected output has no report. A rename that updated one side would
# leave a file the gate never reads.
for expected in "$fixtures_dir"/*.out.json; do
    stem=$(basename "$expected" .out.json)
    if [ ! -f "$reports_dir/$stem.jq" ]; then
        echo "check-jq-reports: $expected has no report at $reports_dir/$stem.jq" >&2
        status=1
    fi
done

exit $status
