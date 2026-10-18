#!/usr/bin/env bash
# Body of `just clippy`. A clippy.toml ban whose path resolves to nothing only
# warns, and `-D warnings` leaves that warning alone, so the ban would silently
# ban nothing. Fail on it.
set -eo pipefail
rc=0
out=$(cargo clippy --workspace --all-targets -q -- -D warnings 2>&1) || rc=$?
[ -z "$out" ] || printf '%s\n' "$out" >&2
if grep -q 'does not refer to a reachable' <<<"$out"; then
    echo "clippy.toml names a disallowed path that resolves to nothing" >&2
    exit 1
fi
exit $rc
