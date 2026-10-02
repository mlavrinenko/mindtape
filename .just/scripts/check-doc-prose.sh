#!/usr/bin/env bash
# .just/scripts/check-doc-prose.sh — the body of the `check-doc-prose` recipe.
# It stops doc prose and config comments from telling history or shouting in
# all-caps words.
#
# It scans the hand-written doc sources, prose and comments alike, and prints
# file, line and word for each hit. It also scans the full-line `#` comments of
# the Justfile, the `.just/*.just` modules and the YAML configs
# `.mmz/config.yaml`, `outdatty.yaml` and `.linecop.yaml`. Code lines in those
# files are not prose and are skipped. The rules it enforces, with examples,
# are in docs/src/contributing/documentation.typ under "Plain prose".
#
# Text inside a raw span, a fenced block or double quotes is skipped, because
# it is code or a quotation: `now()` and "YYYY-MM-DD" never count.
#
# The history list is short on purpose: each entry reads as history wherever
# this tree uses it. "moved", "retired" and "replaced" stay off the list,
# because the docs also use them for what a command does to a task.
#
# An all-caps word passes when it is an acronym in ALLOWED, a constant a Typst
# file declares with `let`, or a file name such as README.md.
set -eo pipefail

ALLOWED="ADR AI ANSI API ASCII ATX CGO CI CLI CPU CQRS CRAP CSS CSV DB DOM DOT
DRY DSL ETA GB GIO GPL GUI HTML ICE ICU ID IEC ISBN ISC ISO JS JSON LLVM MB MCP
MIT OG OS OSC PDF PR PTY README RFC RICE RRULE SEO SGR SHA SQL SSG SVG SWHID
TO-BE TOML TSV TTY TUI UI URI URL UTC UUID VCS WIP WSJF XDG YAML"
HISTORY_WORDS="now anymore formerly"
HISTORY_PHRASES="used to|no longer"

# One awk pass over every file given. Prints "file:line: message" per hit.
scan() {
    local consts
    consts=$(cat "$@" | grep -oE '(^|[^A-Za-z0-9_-])let +[A-Z][A-Z0-9_]*' |
        sed -E 's/.*let +//' | sort -u | tr '\n' ' ' || true)
    awk -v allowed="$ALLOWED $consts" -v words="$HISTORY_WORDS" \
        -v phrases="$HISTORY_PHRASES" '
        BEGIN {
            n = split(allowed, a, /[ \n]+/)
            for (i = 1; i <= n; i++) ok[a[i]] = 1
            nw = split(words, word, / /)
            np = split(phrases, phrase, /\|/)
            for (i = 1; i <= np; i++) {
                re = phrase[i]; gsub(/ /, "[ \t]+", re)
                phrase_re[i] = "(^|[^a-z0-9_])" re "([^a-z0-9_]|$)"
            }
        }
        # The line with raw spans and quoted text blanked out. raw holds the
        # backtick count of the open span; like Typst, a span can wrap lines.
        function visible(line,    out, i, n, c, j, run) {
            out = ""; i = 1; n = length(line)
            while (i <= n) {
                c = substr(line, i, 1)
                if (c == "`") {
                    j = i; while (j <= n && substr(line, j, 1) == "`") j++
                    run = j - i
                    if (raw == 0) { if (run != 2) raw = run }
                    else if (raw == 1 || run == raw) raw = 0
                    out = out " "; i = j; continue
                }
                if (raw) { i++; continue }
                if (c == "\\") { out = out " "; i += 2; continue }
                if (c == "\"") { quoted = !quoted; out = out " "; i++; continue }
                out = out (quoted ? " " : c)
                i++
            }
            return out
        }
        function hit(msg) { printf "%s:%d: %s\n", FILENAME, FNR, msg }
        function count(s, re,    k) {
            k = 0
            while (match(s, re)) { k++; s = substr(s, RSTART + RLENGTH - 1) }
            return k
        }
        FNR == 1 {
            raw = 0; quoted = 0; tail = ""; was_comment = 0
            hash = (FILENAME ~ /(^|\/)Justfile$|\.just$|\.ya?ml$/)
        }
        {
            # In a Justfile or YAML file only a line that starts with `#` is
            # prose. It is read as a `//` comment and every other line as blank.
            if (hash) {
                if ($0 ~ /^[ \t]*#/) sub(/^[ \t]*#+/, "//")
                else $0 = ""
            }
            # An unclosed quote or inline span ends at a blank line, or where
            # comment lines meet code lines. A Justfile or YAML comment has no
            # fenced blocks, so a stray fence ends there too.
            comment = ($0 ~ /^[ \t]*\/\//)
            if ($0 ~ /^[ \t]*$/ || comment != was_comment) {
                quoted = 0
                if (raw == 1 || hash) raw = 0
            }
            was_comment = comment
            v = visible($0)
            sub(/^[ \t]*\/\/+/, "", v)
            low = tolower(v)

            for (i = 1; i <= nw; i++) {
                k = count(low, "(^|[^a-z0-9_])" word[i] "([^a-z0-9_(]|$)")
                while (k-- > 0) hit("history word \"" word[i] "\"")
            }
            # A phrase can wrap, so the last word of the previous line is
            # joined in front of this one.
            for (i = 1; i <= np; i++) {
                k = count(tail " " low, phrase_re[i])
                while (k-- > 0) hit("history word \"" phrase[i] "\"")
            }
            tail = low
            sub(/[^a-z0-9_]+$/, "", tail)
            sub(/.*[^a-z0-9_]/, "", tail)

            s = v
            while (match(s, /[A-Za-z0-9_$][A-Za-z0-9_-]*/)) {
                tok = substr(s, RSTART, RLENGTH)
                s = substr(s, RSTART + RLENGTH)
                if (tok !~ /^[A-Z][A-Z-]*[A-Z]$/ || tok !~ /[A-Z][A-Z]/) continue
                if (tok in ok || s ~ /^\.[a-z]/) continue
                hit("all-caps \"" tok "\"")
            }
        }
    ' "$@"
}

# Control: a scanner that matches nothing passes a dirty tree, so it first has
# to report exactly the hits planted in this fixture.
control() {
    local dir out want
    dir=$(mktemp -d)
    cat >"$dir/f.typ" <<'EOF'
#let NAV = (1,)
This used
to hold. NAV and JSON pass, and so do `now()`, now() and README.md.
// It is now SHOUTED, but "WHY now" is a quotation.
```sh
echo "LOUD used to"
```
A `raw span
now` ends; then no longer, now and now.
EOF
    cat >"$dir/g.yaml" <<'EOF'
key: now used to be fine
# It is now SHOUTED here.
  # A `now` span and "no longer" quote pass.
EOF
    out=$(scan "$dir/f.typ" "$dir/g.yaml" | sed "s|^$dir/||")
    rm -rf "$dir"
    want='f.typ:3: history word "used to"
f.typ:4: history word "now"
f.typ:4: all-caps "SHOUTED"
f.typ:9: history word "now"
f.typ:9: history word "now"
f.typ:9: history word "no longer"
g.yaml:2: history word "now"
g.yaml:2: all-caps "SHOUTED"'
    if [ "$out" != "$want" ]; then
        printf 'check-doc-prose: the control fixture gave\n%s\nexpected\n%s\n' \
            "$out" "$want" >&2
        exit 1
    fi
}

control

shopt -s globstar nullglob
sources=()
for f in docs/src/**/*.typ www/content/*.typ www/utils/*.typ \
    www/templates/*.typ skills/mindtape/references/*.md \
    skills/mindtape/evals/README.md; do
    # tola.typ is tola's own generated helper, never edited here.
    [ "$(basename "$f")" = "tola.typ" ] || sources+=("$f")
done

sources+=(Justfile .just/*.just .mmz/config.yaml outdatty.yaml .linecop.yaml)

hits=$(scan "${sources[@]}")
if [ -n "$hits" ]; then
    sed 's/^/check-doc-prose: /' <<<"$hits" >&2
    echo "check-doc-prose: rewrite each line plainly; the rules are in docs/contributing/documentation.md" >&2
    exit 1
fi
