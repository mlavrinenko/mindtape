#!/usr/bin/env bash
# .just/scripts/plural-check.sh — the body of the `plural-check` recipe
# (truth/tasks/print-counts-as-1-task-or-12-tasks-never-task-s.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# A count reads `1 task` or `12 tasks`, never `task(s)`: `mindtape::count` and
# `mindtape::plural` word it. This refuses `(s)` inside a string literal in
# non-test Rust code under `crates/`, and prints `file:line: <message>` for
# each offender.
#
# What counts as a string and as test code, read line by line with no parser:
#   - A string opens at `"` or `r"`/`r#"`, runs across lines, and honours `\`
#     escapes outside a raw string. A `//` comment, a char literal such as
#     `'"'` and a call such as `width(s)` are not strings.
#   - Test code is a file under `tests/` or `test_support/`, a test sibling
#     (`*_tests*.rs`, `tests_*.rs`, `tests.rs`), and everything from an inline
#     `#[cfg(test)] mod name {` to the end of its file, since the inline test
#     module closes a file by convention.
#
# The fixture. A scan that reaches nothing passes for the wrong reason, so the
# scan runs first over a synthesized tree in a temp dir, holding strings it
# must refuse and every shape it must let through, and only then over
# `crates/`. The temp dir is for size-fixture-check.sh's reason: `just check`
# runs its arms in parallel, and a stray file in the tracked tree would fail
# the arms scanning it.
set -eo pipefail

# scan <dir>: print `file:line: message` for each `(s)` in a string under <dir>.
scan() {
    local file
    while IFS= read -r file; do
        LC_ALL=C awk -v file="$file" '
            function report() {
                printf "%s:%d: (s) in a string; count with mindtape::count(n, \"noun\")\n", file, NR
            }
            BEGIN { in_str = 0; raw = -1; pending = 0 }
            {
                line = $0
                if (!in_str) {
                    if (line ~ /^[[:space:]]*#\[cfg\(test\)\]/) { pending = 1; next }
                    if (pending && line ~ /^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?mod[[:space:]]+[A-Za-z0-9_]+[[:space:]]*\{/) exit
                    if (line !~ /^[[:space:]]*#\[/) pending = 0
                }
                n = length(line); i = 1; hit = 0
                while (i <= n) {
                    c = substr(line, i, 1)
                    if (in_str) {
                        if (raw >= 0) {
                            if (c == "\"" && substr(line, i + 1, raw) == hashes) { in_str = 0; i += raw + 1; continue }
                        } else if (c == "\\") { i += 2; continue }
                        else if (c == "\"") { in_str = 0; i++; continue }
                        if (substr(line, i, 3) == "(s)") hit = 1
                        i++; continue
                    }
                    if (substr(line, i, 2) == "//") break
                    if (c == "\"") { in_str = 1; raw = -1; i++; continue }
                    if (c == "r" && (i == 1 || substr(line, i - 1, 1) !~ /[A-Za-z0-9_]/) && match(substr(line, i), /^r#*"/)) {
                        raw = RLENGTH - 2; hashes = substr(line, i + 1, raw); in_str = 1; i += RLENGTH; continue
                    }
                    if (c == "'\''") {
                        if (substr(line, i, 4) ~ /^'\''\\.'\''/) { i += 4; continue }
                        if (substr(line, i + 2, 1) == "'\''") { i += 3; continue }
                    }
                    i++
                }
                if (hit) report()
            }' "$file"
    done < <(find "$1" -name '*.rs' -not -path '*/target/*' -not -path '*/tests/*' -not -path '*/test_support/*' \
        -not -name '*_tests*.rs' -not -name 'tests_*.rs' -not -name 'tests.rs' -not -name 'test_support.rs' | sort)
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

src="$tmp/crates/x/src"
mkdir -p "$src" "$tmp/crates/x/tests"
cat > "$src/bad.rs" <<'EOF'
fn a(n: usize) { println!("deleted {n} task(s)"); }
fn b() -> &'static str { "first line \
    then 2 file(s)" }
fn c() -> &'static str { r#"a "quoted" link(s)"# }
#[cfg(test)]
mod helper;
fn d() { let _ = "edge(s)"; }
EOF
cat > "$src/ok.rs" <<'EOF'
//! A comment may say task(s).
fn a(s: &str) -> usize { width(s) } // so may this one: file(s)
fn b() -> char { '"' }
fn c(s: &str) -> bool { s == "(" && is(s) }
fn d<'a>(s: &'a str) -> &'a str { trim(s) }
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    const SHOWN: &str = "1 task(s)";
}
EOF
printf 'const X: &str = "task(s)";\n' > "$src/a_tests.rs"
printf 'const X: &str = "task(s)";\n' > "$tmp/crates/x/tests/it.rs"

control=$(scan "$tmp")
expected="$src/bad.rs:1: (s) in a string; count with mindtape::count(n, \"noun\")
$src/bad.rs:3: (s) in a string; count with mindtape::count(n, \"noun\")
$src/bad.rs:4: (s) in a string; count with mindtape::count(n, \"noun\")
$src/bad.rs:7: (s) in a string; count with mindtape::count(n, \"noun\")"
if [ "$control" != "$expected" ]; then
    echo "plural-check: the fixture did not refuse exactly the (s) strings, so the scan proves nothing" >&2
    echo "  got: $control" >&2
    exit 1
fi

offenders=$(scan crates)
if [ -n "$offenders" ]; then
    echo "$offenders" >&2
    echo "plural-check: say 1 task or 12 tasks with mindtape::count or mindtape::plural, not task(s)" >&2
    exit 1
fi
