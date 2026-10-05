#!/usr/bin/env bash
# .just/scripts/docs-md-check.sh — the body of the `md-check` recipe in
# .just/docs.just (invoked as `just docs::md-check`, mangled to `docs-md-check`
# everywhere the docs name it), extracted under truth/tasks/extract-the-long-
# gate-recipe-bodies-into-scripts.typ. Every path below is repo-root-relative:
# that module file carries `set working-directory := '..'` precisely so its
# recipes run at the repo root rather than in .just/. The prose arguing why the
# regenerate goes to a temp directory stays above that recipe.
set -eo pipefail
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
bash docs/generate-md.sh "$tmp"
drift=()
while IFS= read -r -d '' f; do
    rel="${f#"$tmp"/}"
    if [ ! -f "$rel" ] || ! diff -q "$rel" "$f" >/dev/null 2>&1; then
        drift+=("$rel")
    fi
done < <(find "$tmp" -type f -print0)

# Lint: fail on typlite output that GitHub renders broken, which the diff above
# passes because a broken render matches its own committed copy. It checks each
# fresh render outside fenced code and scans docs/src for a `;` glued to a
# `#link(...)[...]`. Each hit prints `file:line: why`; the traps are listed in
# docs/contributing/generated-docs.md.
read -r -d '' lint_md <<'AWK' || true
function bad(why) { printf "%s:%d: %s\n", name, FNR, why; hits++ }
function bare_pipe(s,   i) {
    for (i = 1; i <= length(s); i++)
        if (substr(s, i, 1) == "|" && substr(s, i - 1, 1) != "\\") return 1
    return 0
}
# Where the code span opened by `run` backticks before position j closes, or 0.
function span_end(line, j, run,   k, m) {
    for (k = j; k <= length(line); k++) {
        if (substr(line, k, 1) != "`") continue
        for (m = k; substr(line, m, 1) == "`"; m++) ;
        if (m - k == run) return k
        k = m - 1
    }
    return 0
}
function scan(line, row,   i, j, e, c, n) {
    for (i = 1; i <= length(line); ) {
        c = substr(line, i, 1)
        if (c == "\\") {
            n = substr(line, i + 1, 1)
            if (n == "`") bad("escaped backtick")
            if (n == ">" && substr(line, i - 1, 1) == "-") bad("-\\> from an arrow")
            i += 2
        } else if (c == "`") {
            for (j = i; substr(line, j, 1) == "`"; j++) ;
            e = span_end(line, j, j - i)
            if (!e) { bad("backtick with no closing backtick on the line"); i = j; continue }
            if (row && bare_pipe(substr(line, j, e - j))) bad("| inside a code span in a table row")
            for (i = e; substr(line, i, 1) == "`"; i++) ;
        } else i++
    }
}
{ line = $0; sub(/^ */, "", line) }
# Closing fence: only fence characters, at least as many as the opening.
fchar != "" {
    if (line ~ /^(```+|~~~+) *$/ && substr(line, 1, 1) == fchar) {
        sub(/ *$/, "", line)
        if (length(line) >= flen) fchar = ""
    }
    next
}
line ~ /^(```+|~~~+)/ { fchar = substr(line, 1, 1); match(line, /^[`~]+/); flen = RLENGTH; next }
{ scan($0, $0 ~ /^ *\|/) }
END { exit hits > 0 }
AWK
read -r -d '' lint_src <<'AWK' || true
{ buf = buf $0 "\n" }
END {
    for (off = 0; match(substr(buf, off + 1), /#link\("[^"]*"\)\[[^]]*\];/); off += RSTART + RLENGTH - 1) {
        head = substr(buf, 1, off + RSTART + RLENGTH - 1)
        printf "%s:%d: ; glued to a #link(...)[...] is swallowed\n", name, gsub(/\n/, "\n", head) + 1
        hits++
    }
    exit hits > 0
}
AWK
lint_hits=()
while IFS= read -r -d '' f; do
    rel="${f#"$tmp"/}"
    awk -v name="$rel" "$lint_md" "$f" >&2 || lint_hits+=("$rel")
done < <(find "$tmp" -type f -name '*.md' -print0)
while IFS= read -r -d '' f; do
    awk -v name="$f" "$lint_src" "$f" >&2 || lint_hits+=("$f")
done < <(find docs/src -type f -name '*.typ' -print0)

if [ "${#drift[@]}" -gt 0 ]; then
    echo "docs::md-check: drifted from docs/src/ (run \`just docs::md\` to refresh):" >&2
    for f in "${drift[@]}"; do echo "  - $f" >&2; done
fi
if [ "${#lint_hits[@]}" -gt 0 ]; then
    echo "docs::md-check: the hits above are typlite traps; see docs/contributing/generated-docs.md" >&2
fi
if [ "${#drift[@]}" -gt 0 ] || [ "${#lint_hits[@]}" -gt 0 ]; then
    exit 1
fi
