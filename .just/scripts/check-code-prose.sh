#!/usr/bin/env bash
# .just/scripts/check-code-prose.sh — the body of the `lint::vale-code`
# recipe. Runs from the repo root, like the recipe that invokes it.
#
# Vale reads comments through the Rust grammar. .config/vale-code.ini names
# the RustDoc View, a tree-sitter query that keeps `///` and `//!` and drops an
# inline `//` comment. `[formats] rs = md` parses the comment body as Markdown,
# so a code span and a fenced doc test stay out. This script points Vale at
# every Rust source under `crates/`.
#
# The control proves the View selects a doc comment, drops an inline one, and
# keeps each line number.
set -eo pipefail

# The control fixture. A hit in each doc form, and a decoy in each form the
# View must skip.
control() {
    local dir actual expected
    dir=$(mktemp -d)
    cat >"$dir/demo.rs" <<'EOF'
//! The crate used to say this.
// Inline used to stay out.
//// Plain used to stay out.
/// A used to phrase in a doc line.
    /// An indented used to phrase.
/// ```
/// used to stay out.
/// ```
fn code() { let s = "used to"; }
/// A `used to` span passes.
/// A SHOUTED word fails.
/// A `SHOUTED` span passes.
EOF
    set +e
    actual=$(vale --no-global --config=.config/vale-code.ini --output=line "$dir/demo.rs" 2>&1)
    set -e
    rm -rf "$dir"
    actual=$(printf '%s\n' "$actual" |
        sed -E "s|^$dir/||; s|^([^:]+):([0-9]+):[0-9]+:([A-Za-z.]+):.*|\1:\2:\3|")
    expected='demo.rs:1:Prose.History
demo.rs:4:Prose.History
demo.rs:5:Prose.History
demo.rs:11:Prose.Shouting'
    if [ "$actual" != "$expected" ]; then
        printf 'check-code-prose: the control fixture gave\n%s\nexpected\n%s\n' \
            "$actual" "$expected" >&2
        exit 1
    fi
}

control

git ls-files -z --cached --others --exclude-standard -- 'crates/**/*.rs' |
    xargs -0 -r vale --no-global --config=.config/vale-code.ini --output=line
