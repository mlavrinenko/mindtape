#!/usr/bin/env bash
# .just/scripts/check-schema-urls.sh — the schema URL arm of `docs::check`.
# Runs from the repo root, like the recipe that invokes it.
#
# Every registered output has a committed schema under schema/output/, and
# every raw.githubusercontent.com URL under schema/output/ in the docs sources
# or their captures names one of those files. The first arm keeps the copy
# `mt install` writes complete; the second catches a page or a fixture that
# names a version the tree never recorded. `mt schema --check` holds the bytes;
# this holds only that the file exists.
#
# A `file://` base, which a working-tree build names when
# MINDTAPE_SCHEMA_BASE points at the checkout, and the JSON Schema draft URL are
# not ours, so neither is a finding: the scan only matches the canonical
# raw.githubusercontent.com base.
set -eo pipefail

# Read www/generated/ under the lock every docs reader takes: a concurrent
# docs::gen wipes and recreates it, and half a directory is worse than none.
# A caller may already hold the lock (www/generate.sh's contract), so the
# handshake skips a second flock on this process's own descriptor.
mkdir -p target
if [ -z "${MINDTAPE_GENERATED_LOCK:-}" ]; then
    exec 9>target/www-generated.lock
    flock 9
    export MINDTAPE_GENERATED_LOCK=1
fi

fail=0
schemas=$(./target/debug/mt schema) || {
    echo "check-schema-urls: ./target/debug/mt is missing; build it or run just docs::gen" >&2
    exit 1
}

while read -r name version _url; do
    path="schema/output/$name/$version.schema.json"
    if [ ! -f "$path" ]; then
        echo "check-schema-urls: mt schema lists $name $version, but $path is missing" >&2
        fail=1
    fi
done <<<"$schemas"

urls=$(grep -rhoE 'https://raw\.githubusercontent\.com/[^[:space:]"<>]+/schema/output/[^[:space:]"<>]+\.schema\.json' \
    www/content www/generated 2>/dev/null | sort -u)
while IFS= read -r url; do
    [ -n "$url" ] || continue
    path="schema/output/${url##*/schema/output/}"
    if [ ! -f "$path" ]; then
        echo "check-schema-urls: $url names $path, which is missing" >&2
        fail=1
    fi
done <<<"$urls"

if [ "$fail" -ne 0 ]; then
    echo "check-schema-urls: run mt schema --write, or point the doc at a recorded version" >&2
fi
exit "$fail"
