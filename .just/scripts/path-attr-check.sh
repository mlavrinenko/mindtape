#!/usr/bin/env bash
# .just/scripts/path-attr-check.sh — the body of the `path-attr-check` recipe
# (truth/tasks/hold-the-folder-per-area-layout-with-a-lint-and-a-path-check.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# A module's path is its file path: an area is a folder whose `mod.rs` lists
# its files, and `#[path = "..."]` names only a test sibling (`*_tests*.rs`,
# `tests_*.rs`, `tests.rs`). `self_named_module_files` in the workspace lints
# refuses `foo.rs` beside `foo/`; it cannot see a `#[path]`, and the line cap
# pushes a split straight back to a flat `#[path]` file at the crate root. This
# is the other half. It prints `file:line: <message>` for each offender.
#
# The fixture. A check that scans an empty or unreachable tree passes for the
# wrong reason, so the scan runs first over a synthesized tree in a temp dir —
# stray `#[path]` attributes that must be refused, and every allowed shape that
# must not — and only then over `crates/`. The fixture lives in a temp dir for
# size-fixture-check.sh's reason: `just check` runs its arms in parallel, and a
# stray file in the tracked tree would fail the arms scanning it.
set -eo pipefail

# scan <dir>: print `file:line: message` for each refused `#[path]` under <dir>.
scan() {
    local file
    while IFS= read -r file; do
        awk -v file="$file" '
            function allowed(target,    base) {
                base = target; sub(/.*\//, "", base)
                return base ~ /_tests[^\/]*\.rs$/ || base ~ /^tests_[^\/]*\.rs$/ || base == "tests.rs"
            }
            match($0, /^[[:space:]]*#\[(cfg_attr\([^]]*,[[:space:]]*)?path[[:space:]]*=[[:space:]]*"[^"]*"/) {
                target = substr($0, RSTART, RLENGTH); sub(/^[^"]*"/, "", target); sub(/"$/, "", target)
                if (!allowed(target))
                    printf "%s:%d: #[path = \"%s\"] names no test sibling; make the module a file path (foo/mod.rs, foo/bar.rs)\n", file, NR, target
            }' "$file"
    done < <(find "$1" -name '*.rs' -not -path '*/target/*' | sort)
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/crates/x/src"
printf '#[cfg(test)]\n#[path = "a_tests.rs"]\nmod t;\n#[path = "tests_b.rs"]\nmod u;\n#[path = "tests.rs"]\nmod v;\n#[path = "a_tests_c.rs"]\nmod w;\n' > "$tmp/crates/x/src/ok.rs"
printf '//! `#[path]` in a doc line is not an attribute.\n' > "$tmp/crates/x/src/doc.rs"
printf '#[cfg_attr(unix, path = "unix_impl.rs")]\nmod imp;\n' > "$tmp/crates/x/src/other.rs"
printf 'mod a;\n#[path = "flat_new.rs"]\nmod new;\n' > "$tmp/crates/x/src/stray.rs"

control=$(scan "$tmp")
expected="$tmp/crates/x/src/other.rs:1: #[path = \"unix_impl.rs\"] names no test sibling; make the module a file path (foo/mod.rs, foo/bar.rs)
$tmp/crates/x/src/stray.rs:2: #[path = \"flat_new.rs\"] names no test sibling; make the module a file path (foo/mod.rs, foo/bar.rs)"
if [ "$control" != "$expected" ]; then
    echo "path-attr-check: the fixture did not refuse exactly the stray #[path] attributes — the scan proves nothing" >&2
    echo "  got: $control" >&2
    exit 1
fi

offenders=$(scan crates)
if [ -n "$offenders" ]; then
    echo "$offenders" >&2
    echo "path-attr-check: #[path] may name only a test sibling" >&2
    exit 1
fi
