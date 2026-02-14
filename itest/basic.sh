#!/usr/bin/env bash
set -euo pipefail

PASS=0
FAIL=0

check() {
    local name="$1"
    local expected="$2"
    local actual="$3"
    if [ "$actual" = "$expected" ]; then
        PASS=$((PASS + 1))
    else
        FAIL=$((FAIL + 1))
        echo "FAIL: $name"
        echo "  expected: $expected"
        echo "  actual:   $actual"
    fi
}

# --- Eval command (backwards compat) ---

output=$(mindtape res/piano.typ --due -2)
expected_output="- (due 2026-04-01) Learn 5 Hanon exercises
- (due 2026-05-02) Finish learning Lilium"
check "eval --due -2" "$expected_output" "$output"

# --- Query commands: build index, then query ---

TMPDIR=$(mktemp -d)
DB="$TMPDIR/test.db"
trap 'rm -rf "$TMPDIR"' EXIT

# Create a config pointing the DB to our temp dir, then run watch briefly to index.
cat > "$TMPDIR/config.toml" <<EOF
[database]
path = "$DB"

[[watch]]
path = "res"
EOF

mindtape watch --config "$TMPDIR/config.toml" &
WATCH_PID=$!
sleep 1
kill $WATCH_PID 2>/dev/null || true
wait $WATCH_PID 2>/dev/null || true

# Now query the DB
output=$(mindtape list --status all --db "$DB" 2>/dev/null)
check "list --status all contains tasks" "true" "$(echo "$output" | grep -q 'Learn 5 Hanon' && echo true || echo false)"

output=$(mindtape list --db "$DB" 2>/dev/null)
check "list default shows pending" "true" "$(echo "$output" | grep -q 'Pick a song' && echo true || echo false)"
check "list default hides done" "false" "$(echo "$output" | grep -q "Finish learning Shining" && echo true || echo false)"

output=$(mindtape status --db "$DB")
check "status shows files" "true" "$(echo "$output" | grep -q 'files:' && echo true || echo false)"
check "status shows tasks" "true" "$(echo "$output" | grep -q 'tasks:' && echo true || echo false)"

output=$(mindtape files --db "$DB")
check "files lists piano.typ" "true" "$(echo "$output" | grep -q 'piano.typ' && echo true || echo false)"

# --- Error: query without DB ---

if mindtape list --db "$TMPDIR/nonexistent.db" 2>/dev/null; then
    FAIL=$((FAIL + 1))
    echo "FAIL: list with missing DB should fail"
else
    PASS=$((PASS + 1))
fi

# --- Summary ---

echo ""
echo "$PASS passed, $FAIL failed"
if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
