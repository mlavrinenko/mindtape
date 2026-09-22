#!/usr/bin/env bash
# Body of `just outdatty-update GROUP FILE`: record FILE's hash in GROUP and
# nothing else. outdatty 0.4 scopes `update` to a group at the finest, so this
# runs the group update and then puts back every lock line it changed except
# FILE's — the other pending dependents stay pending, and the commit's lock
# diff names only the file its author reviewed. Why, in outdatty.yaml's header.
set -euo pipefail
GROUP=$1
FILE=$2
LOCK=outdatty.lock
before=$(mktemp)
after=$(mktemp)
trap 'rm -f "$before" "$after"' EXIT
cp "$LOCK" "$before"
outdatty update --group "$GROUP" --format quiet
# `update` rewrites hashes in place; a changed line count means the manifest
# added or dropped entries, which is a group re-record, not a one-file one.
if [ "$(wc -l <"$before")" != "$(wc -l <"$LOCK")" ]; then
  cp "$before" "$LOCK"
  echo "outdatty-update: $GROUP's entries changed shape; run 'just outdatty-update $GROUP' and review the whole group" >&2
  exit 1
fi
awk -v file="$FILE" '
  NR == FNR { old[FNR] = $0; next }
  { key = $1; sub(/:$/, "", key) }
  key == file { hit = 1; print; next }
  { print old[FNR] }
  END { exit !hit }
' "$before" "$LOCK" >"$after" || {
  cp "$before" "$LOCK"
  echo "outdatty-update: $FILE is not in the lock; name a path outdatty.yaml's $GROUP lists" >&2
  exit 1
}
cp "$after" "$LOCK"
