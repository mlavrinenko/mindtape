#!/usr/bin/env bash
# .just/scripts/_crap-ratchet-control.sh — the control `just crap::check` runs before
# it scores the real tree. Runs from the repo root. Builds a one-file tree with
# two same-named functions in a temp dir, never the working tree.
set -eo pipefail

# shellcheck source=.just/scripts/_crap-ratchet-lib.sh
source .just/scripts/_crap-ratchet-lib.sh

fail_control() {
    echo "crap: control failed: $*" >&2
    exit 1
}

expect() { # <label> <want: 0|1> <output-file> <rc> <needle>...
    local label=$1 want=$2 file=$3 rc=$4 needle
    shift 4
    [ "$rc" -eq "$want" ] || fail_control "$label: exit $rc, want $want: $(head -c 900 "$file")"
    for needle in "$@"; do
        grep -qF -- "$needle" "$file" || fail_control "$label: did not name $needle: $(head -c 900 "$file")"
    done
}

# write_tree <dir> <blank lines above the impls>: two `D::from`, CC 1 and 3,
# with one line of the second left uncovered.
write_tree() {
    local d=$1 pad=$2 i
    mkdir -p "$d/src"
    {
        echo "pub struct D(pub u8);"
        for ((i = 0; i < pad; i++)); do echo; done
        cat <<'RS'
impl From<u8> for D {
    fn from(n: u8) -> Self {
        D(n)
    }
}
impl From<i8> for D {
    fn from(n: i8) -> Self {
        if n > 0 {
            D(1)
        } else if n < -5 {
            D(2)
        } else {
            D(3)
        }
    }
}
RS
    } > "$d/src/lib.rs"
    {
        echo "SF:$d/src/lib.rs"
        for i in 3 4 8 9 10 11 13 14; do echo "DA:$((i + pad)),1"; done
        echo "DA:$((12 + pad)),0"
        echo "end_of_record"
    } > "$d/lcov.info"
}

# mint <dir> <out>: what crap-baseline.sh commits, root-relative paths.
mint() {
    cargo crap --path "$1" --lcov "$1/lcov.info" --format json --sort file --output "$2.raw"
    jq --arg root "$1/" '.entries |= map(.file |= ltrimstr($root))' "$2.raw" > "$2"
}

ratchet() { # <dir> <baseline> <out>; prints the exit status
    local rc=0
    crap_ratchet "$1" "$2" --path "$1" --lcov "$1/lcov.info" > "$3" 2>&1 || rc=$?
    echo "$rc"
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
write_tree "$tmp/a" 0
mint "$tmp/a" "$tmp/base.json"

# A file that only shifted pairs every same-named function with itself.
write_tree "$tmp/b" 3
cp "$tmp/base.json" "$tmp/b.json"
rc=$(ratchet "$tmp/b" "$tmp/b.json" "$tmp/out")
expect "shifted, unchanged" 0 "$tmp/out" "$rc" "0 regressed" "0 new" "0 removed"

# A rise in the second of two same-named functions fails, naming its line.
jq '.entries |= map(if .line == 8 then .crap = 2 else . end)' "$tmp/base.json" > "$tmp/c.json"
rc=$(ratchet "$tmp/b" "$tmp/c.json" "$tmp/out")
expect "shifted, second regressed" 1 "$tmp/out" "$rc" "1 regressed" "src/lib.rs:11"

# A function the baseline does not list fails, naming it and the re-mint.
jq '.entries |= map(select(.line != 8))' "$tmp/base.json" > "$tmp/d.json"
rc=$(ratchet "$tmp/b" "$tmp/d.json" "$tmp/out")
expect "shifted, second unlisted" 1 "$tmp/out" "$rc" "no baseline entry" "src/lib.rs:11 D::from" "just crap::baseline"
