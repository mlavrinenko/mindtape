#!/usr/bin/env bash
# .just/scripts/check-code-prose.sh: the body of the `lint::vale-code`
# recipe. Runs from the repo root, like the recipe that invokes it.
#
# Vale reads a comment through its file's grammar. The RustDoc View keeps
# `///` and `//!` and drops an inline `//` comment. `[formats] rs = md` parses
# the comment body as Markdown, so a code span and a fenced doc test stay out.
# Vale reads a shell script through the R grammar, whose Comments View keeps
# each `#` comment and drops a shebang, a heredoc body and a `shellcheck`
# directive. This script points Vale at every Rust source under `crates/`
# and every tracked shell script.
#
# The control proves each reader picks the lines it should and keeps each
# line number.
#
set -eo pipefail

# The control fixtures. A hit in each comment form, and a decoy in each form
# the reader must skip.
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
    cat >"$dir/demo.sh" <<'SH'
#!/usr/bin/env bash
# A used to phrase in a comment.
# A `used to` span passes.
cat <<'EOF'
used to stays out of the heredoc.
EOF
# shellcheck source=/dev/null
# A SHOUTED word fails.
echo "used to stays out of code."
SH
    set +e
    actual=$(vale --no-global --config=.config/vale-code.ini --output=line "$dir" 2>&1)
    set -e
    rm -rf "$dir"
    actual=$(printf '%s\n' "$actual" |
        sed -E "s|^$dir/||; s|^([^:]+):([0-9]+):[0-9]+:([A-Za-z.]+):.*|\1:\2:\3|")
    expected='demo.rs:1:Prose.History
demo.rs:4:Prose.History
demo.rs:5:Prose.History
demo.rs:11:Prose.Shouting
demo.sh:2:Prose.History
demo.sh:8:Prose.Shouting'
    if [ "$actual" != "$expected" ]; then
        printf 'check-code-prose: the control fixture gave\n%s\nexpected\n%s\n' \
            "$actual" "$expected" >&2
        exit 1
    fi
}

control

git ls-files -z --cached --others --exclude-standard -- 'crates/**/*.rs' '*.sh' |
    xargs -0 -r vale --no-global --config=.config/vale-code.ini --output=line
