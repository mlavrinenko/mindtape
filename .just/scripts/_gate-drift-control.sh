#!/usr/bin/env bash
# .just/scripts/_gate-drift-control.sh — the control `just meta::gate-drift`
# runs before it reads the real tree (see check-gate-drift.sh for why a control
# at all). Runs from the repo root. Builds synthetic trees in a temp dir, never
# the working tree: `just check` runs its arms in parallel, so a probe in the
# tracked tree would fail a concurrent gate.
set -eo pipefail

# shellcheck source=.just/scripts/_gate-drift-lib.sh
source .just/scripts/_gate-drift-lib.sh

control() {
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    control_toml "$tmp"
    control_build "$tmp"
}

fail_control() {
    echo "check-gate-drift: control failed: $*" >&2
    exit 1
}

# One run, asserting exit status and that every needle is named in its output
# and no `forbid` is.
expect() { # <label> <want: 0|1> <output-file> <rc> <needle>... ; `NOT:x` forbids
    local label=$1 want=$2 file=$3 rc=$4 needle
    shift 4
    [ "$rc" -eq "$want" ] || fail_control "$label: exit $rc, want $want: $(head -c 700 "$file")"
    for needle in "$@"; do
        if [ "${needle#NOT:}" != "$needle" ]; then
            ! grep -qF -- "${needle#NOT:}" "$file" || fail_control "$label: named ${needle#NOT:}: $(head -c 700 "$file")"
        else
            grep -qF -- "$needle" "$file" || fail_control "$label: did not name $needle: $(head -c 700 "$file")"
        fi
    done
}

control_toml() {
    local d=$1/toml rc=0
    mkdir -p "$d/sub"
    git -C "$d" init -q
    printf 'TOML_FMT_PATHS := "a.toml sub/b.toml"\n' >"$d/Justfile"
    : >"$d/a.toml"
    : >"$d/sub/b.toml"
    printf 'ignored.toml\n' >"$d/.gitignore"
    : >"$d/ignored.toml"
    git -C "$d" add a.toml sub/b.toml Justfile .gitignore
    toml_drift "$d" 2>"$d/out" || rc=$?
    expect "toml positive control" 0 "$d/out" "$rc"
    # Staged-but-unlisted, new-and-unadded, and listed-but-gone, together; the
    # ignored file must stay unreported.
    : >"$d/staged.toml"
    git -C "$d" add staged.toml
    : >"$d/sub/new.toml"
    printf 'TOML_FMT_PATHS := "a.toml sub/b.toml gone.toml"\n' >"$d/Justfile"
    rc=0
    toml_drift "$d" 2>"$d/out" || rc=$?
    expect "toml drift" 1 "$d/out" "$rc" \
        "missing (git knows it, the list lacks it): staged.toml" \
        "missing (git knows it, the list lacks it): sub/new.toml" \
        "extra (the list names it, git does not know it): gone.toml" \
        "NOT:ignored.toml" "NOT:a.toml" "NOT:sub/b.toml"
}

control_build() {
    local d=$1/build rc=0
    mkdir -p "$d/clean/.just/scripts" "$d/clean/www"
    write_clean "$d/clean"
    build_drift "$d/clean" 2>"$d/clean.out" || rc=$?
    expect "build positive control" 0 "$d/clean.out" "$rc"
    cp -r "$d/clean" "$d/bad"
    write_bad "$d/bad"
    rc=0
    build_drift "$d/bad" 2>"$d/bad.out" || rc=$?
    expect "build drift" 1 "$d/bad.out" "$rc" \
        "bad-recipe body line 1" \
        "bad-recipe body line 2" \
        "bad-recipe body line 3" \
        "bad-recipe body line 4" \
        "bad-nested body line 1" \
        ".just/scripts/bad-script.sh:3" \
        "www/inner.sh:2" \
        ".just/scripts/bad-cont.sh:2" \
        "NOT:bad-recipe body line 5" \
        "NOT:ok-script" "NOT:unreached" "NOT:clean-arm" "NOT:orphan-run"
}

# Every shape that must not be a finding. `unreached` and `orphan-run` are a
# plain build and a plain run in recipes `check` never reaches.
write_clean() {
    cat >"$1/Justfile" <<'EOF'
check: (mmz "clean-arm") (mmz "ok-script")

mmz *ARGS:
    true

clean-arm:
    cargo build -p mindtape-cli --tests -q
    cargo build --release -p mindtape-cli
    CARGO_TARGET_DIR=target/x cargo build -p mindtape-cli
    cargo build -p some-other-crate
    cargo build -p mindtape -q
    cargo clippy --workspace --all-targets -q
    cargo nextest run --workspace
    echo "cargo build -p mindtape-cli" # a hint, not a call
    cargo run -p mindtape-cli --target-dir target/y -- check

ok-script:
    bash .just/scripts/ok-script.sh

unreached:
    cargo build -p mindtape-cli -q

orphan-run:
    cargo run -p mindtape-cli -q -- check
EOF
    cat >"$1/.just/scripts/ok-script.sh" <<'EOF'
#!/usr/bin/env bash
# cargo build -p mindtape-cli -q
cargo build --workspace --tests -q
cargo build --target-dir target/z -p mindtape-cli
cat <<'EOT'
cargo build -p mindtape-cli -q
EOT
EOF
}

write_bad() {
    cat >>"$1/Justfile" <<'EOF'

bad-recipe:
    cargo build -p mindtape-cli -q
    cargo build --workspace
    cargo build
    cargo run -p mindtape-cli -q -- check --tests
    cargo build -p mindtape-cli --tests

bad-nested:
    cargo build --package mindtape-cli

bad-via-just:
    just bad-nested
EOF
    sed -i 's/^check: .*/check: (mmz "clean-arm") (mmz "ok-script") (mmz "bad-recipe") (mmz "bad-via-just") (mmz "bad-script") (mmz "bad-www") (mmz "bad-cont")/' "$1/Justfile"
    cat >>"$1/Justfile" <<'EOF'

bad-script:
    bash .just/scripts/bad-script.sh

bad-www:
    bash www/outer.sh

bad-cont:
    bash .just/scripts/bad-cont.sh
EOF
    printf '#!/usr/bin/env bash\nset -e\ncargo build -p mindtape-cli -q\n' >"$1/.just/scripts/bad-script.sh"
    # shellcheck disable=SC2016 # the fixture script must hold a literal $SCRIPT_DIR
    printf '#!/usr/bin/env bash\nbash "$SCRIPT_DIR/inner.sh"\n' >"$1/www/outer.sh"
    printf '#!/usr/bin/env bash\ncargo run -p mindtape-cli -q -- check\n' >"$1/www/inner.sh"
    printf '#!/usr/bin/env bash\ncargo build \\\n    -p mindtape-cli -q\n' >"$1/.just/scripts/bad-cont.sh"
}


control
