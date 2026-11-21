#!/usr/bin/env bash
# .just/scripts/check-no-just-in-output.sh: the body of the
# `lint::no-just-in-output` recipe
# (truth/tasks/2026-11/refuse-a-repo-local-recipe-name-in-the-clis-output.typ).
# Runs from the repo root, like the recipe that invokes it.
#
# A user-facing string says `mt check`, never `just check`. `mt` installs into
# projects that have no Justfile, so a hint to run `just check` sends the reader
# to a command their project lacks. This refuses such a hint in the strings the
# binary prints.
#
# What counts as user-facing, read line by line with no parser:
#   - every non-comment line of every `*.rs` under `crates/`.
#   - A `//`, `///` or `//!` line is a comment, and so is a line-leading
#     `/* ... */` block. A doc comment never reaches a user: a `Diag` hint is a
#     string literal. Clap's help comes from `///` lines, so
#     `crates/mindtape-cli/tests/it/cli_text_e2e.rs` reads the help back out of
#     the binary and catches them there.
#   - Test code is not user-facing: a file under `tests/` or `test_support/`, a
#     test sibling (`*_tests*.rs`, `tests_*.rs`, `tests.rs`, `test_support.rs`),
#     and an inline `#[cfg(test)] mod name {` to the end of its file. That is
#     the definition plural-check.sh already uses.
#
# The Justfile supplies the recipe names: every `namepath` in
# `just --dump --dump-format json`, with modules walked recursively. So
# `just lint::size` gets read, and a renamed recipe cannot slip through. A
# module's bare name (`just docs`) is no recipe and stays out of the match:
# `install`, `docs` and `bench` are ordinary English words, and a gate that
# failed on "just install the file" would get turned off.
#
# The control runs first, over synthetic trees in a temp dir (never the working
# tree: `just check` runs its arms in parallel). It requires a clean tree to
# pass and a bad tree to fail, with the file, line and recipe stated. It also
# reads a tree whose Justfile declares `verify`: that tree catches
# `just verify` and lets `just check` through. The last tree proves the scan
# reads the recipe list from the dump instead of knowing it.
#
# Static: the scan reads text only.
set -eo pipefail

# `names_from` <dir>: every `namepath` the Justfile in <dir> declares, one per
# line, longest first so a name that extends another wins the match.
names_from() {
    (cd "$1" && just --dump --dump-format json) | jq -r '
        def all_recipes:
          ((.recipes // {}) | to_entries | map(.value))
          + ((.modules // {}) | to_entries | map(.value) | map(all_recipes) | add // []);
        [all_recipes[] | .namepath] | unique | .[]' | LC_ALL=C sort -r
}

# scan <rust-root> <just-root>: print one `error: <file>:<line>: ...` line per
# user-facing mention under <rust-root>, reading the recipe names from
# <just-root>/Justfile.
scan() {
    local rust_root=$1 just_root=$2 names file
    if [ ! -d "$rust_root" ]; then
        echo "check-no-just-in-output: no $rust_root directory — running outside the repo root?" >&2
        return 1
    fi
    names=$(names_from "$just_root") || {
        echo "check-no-just-in-output: cannot read the recipe names from $just_root/Justfile" >&2
        return 1
    }
    if [ -z "$names" ]; then
        echo "check-no-just-in-output: $just_root/Justfile declares no recipe" >&2
        return 1
    fi
    while IFS= read -r file; do
        LC_ALL=C awk -v file="$file" -v names="$names" '
            BEGIN { n = split(names, raw, "\n") }
            # mentions: a `just` word, whitespace, then exactly this recipe name
            # ending at a non-name character. String functions, not a built
            # regex: a recipe name is data and needs no escaping.
            function mentions(line, name,   pos, p, before, after, rest) {
                pos = 1
                while ((p = index(substr(line, pos), "just")) > 0) {
                    p += pos - 1
                    before = (p > 1) ? substr(line, p - 1, 1) : ""
                    after = substr(line, p + 4)
                    if (before !~ /[[:alnum:]_]/ && after ~ /^[[:space:]]+/) {
                        rest = after
                        sub(/^[[:space:]]+/, "", rest)
                        if (index(rest, name) == 1 && substr(rest, length(name) + 1, 1) !~ /[[:alnum:]_:-]/) return 1
                    }
                    pos = p + 4
                }
                return 0
            }
            function report(name) {
                printf "error: %s:%d: names the recipe `%s`, which only this repo has\n", file, FNR, name
            }
            {
                if (in_block) { if (index($0, "*/") > 0) in_block = 0; next }
                if ($0 ~ /^[[:space:]]*\/\*/) { if (index($0, "*/") == 0) in_block = 1; next }
                if ($0 ~ /^[[:space:]]*\/\//) next
                if ($0 ~ /^[[:space:]]*#\[cfg\(test\)\]/) { pending = 1; next }
                if (pending && $0 ~ /^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?mod[[:space:]]+[A-Za-z0-9_]+[[:space:]]*\{/) exit
                if (pending && $0 !~ /^[[:space:]]*#\[/) pending = 0
                for (i = 1; i <= n; i++) {
                    if (mentions($0, raw[i])) { report(raw[i]); break }
                }
            }' "$file"
    done < <(find "$rust_root" -name '*.rs' -not -path '*/target/*' -not -path '*/tests/*' \
        -not -path '*/test_support/*' -not -name '*_tests*.rs' -not -name 'tests_*.rs' \
        -not -name 'tests.rs' -not -name 'test_support.rs' | LC_ALL=C sort)
}

# fixture <dir>: a Justfile with one recipe and one module recipe, plus the
# crates/ root the tree's Rust files go under.
fixture() {
    local d=$1
    mkdir -p "$d/.just" "$d/crates/x/src" "$d/crates/x/tests"
    printf "mod lint '.just/lint.just'\ncheck:\n    true\n" >"$d/Justfile"
    printf 'size:\n    true\n' >"$d/.just/lint.just"
}

fail() {
    echo "check-no-just-in-output: control failed: $*" >&2
    echo "  the scan below proves nothing until the control passes" >&2
    exit 1
}

control() {
    local d expected got

    # Clean: every comment shape, a non-recipe word, test code in all three
    # places, and no user-facing mention.
    d=$tmp/clean
    fixture "$d"
    cat >"$d/crates/x/src/ok.rs" <<'EOF'
//! A doc comment may name `just check` and `just lint::size`.
/// A doc item may name `just check` too.
// An ordinary comment may name `just lint::size`.
/* A block comment may name just check. */
fn english() -> &'static str { "not just anything, and not adjust checks" }
fn longer_word() -> &'static str { "just checkpoints, not a recipe" }
#[cfg(test)]
mod tests {
    const A: &str = "run just check";
    const B: &str = "run just lint::size";
}
EOF
    printf 'const A: &str = "run just check";\n' >"$d/crates/x/src/ok_tests.rs"
    printf 'const A: &str = "run just check";\n' >"$d/crates/x/tests/it.rs"
    got=$(scan "$d/crates" "$d") || fail "clean tree: the scan itself failed"
    [ -z "$got" ] || fail "clean tree: scan printed [$got]"

    # Bad: the comments and test code around two real mentions must not hide
    # them, and the mention order must not reorder the output.
    d=$tmp/bad
    fixture "$d"
    cat >"$d/crates/x/src/bad.rs" <<'EOF'
// a comment may name just check
//! a doc comment may name just lint::size
const A: &str = "run just check";
const B: &str = "then just lint::size";
const C: &str = "just checks, not check";
const D: &str = "adjust check";
EOF
    expected="error: $d/crates/x/src/bad.rs:3: names the recipe \`check\`, which only this repo has
error: $d/crates/x/src/bad.rs:4: names the recipe \`lint::size\`, which only this repo has"
    got=$(scan "$d/crates" "$d") || fail "bad tree: the scan itself failed"
    [ "$got" = "$expected" ] || fail "bad tree: got [$got], want [$expected]"

    # Renamed: the same two lines, read against a Justfile that declares
    # `verify` and not `check`. A reader with a hand list would fail here.
    d=$tmp/renamed
    mkdir -p "$d/crates/x/src"
    printf 'verify:\n    true\n' >"$d/Justfile"
    cat >"$d/crates/x/src/renamed.rs" <<'EOF'
const OLD: &str = "run just check";
const NEW: &str = "run just verify";
EOF
    expected="error: $d/crates/x/src/renamed.rs:2: names the recipe \`verify\`, which only this repo has"
    got=$(scan "$d/crates" "$d") || fail "renamed tree: the scan itself failed"
    [ "$got" = "$expected" ] || fail "renamed tree: got [$got], want [$expected]"
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

control

offenders=$(scan crates .)
if [ -n "$offenders" ]; then
    printf '%s\n' "$offenders" >&2
    echo "check-no-just-in-output: a user-facing string names a recipe only this repo has" >&2
    echo "  name a command mt provides instead: \`mt check\`, not \`just check\`" >&2
    exit 1
fi
