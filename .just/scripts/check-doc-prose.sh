#!/usr/bin/env bash
# .just/scripts/check-doc-prose.sh — the body of the `check-doc-prose` recipe.
# It stops doc prose, config comments and config schema descriptions from
# telling history or shouting in all-caps words.
#
# It scans the hand-written doc sources, prose and comments alike, and prints
# file, line and word for each hit. The Typst sources come through the shared
# extractor in _doc-blocks-lib.sh, which yields each line of rendered text and
# each comment line apart from raw spans, fenced blocks and code. Markdown
# sources have no Typst parse tree, so their lines are read as written. It also scans the full-line `#` comments of
# the Justfile, the `.just/*.just` modules and the YAML configs
# `.mmz/config.yaml`, `outdatty.yaml` and `.linecop.yaml`. Code lines in those
# files are not prose and are skipped. It reads every `description` out of
# schema/config.schema.json too, because those texts reach the docs site and
# editors, and a hit there names the key path. The rules it enforces, with
# examples, are in docs/src/contributing/documentation.typ under "Plain prose".
#
# Text inside a raw span, a fenced block or double quotes is skipped, because
# it is code or a quotation: `now()` and "YYYY-MM-DD" never count. A comment
# line and a Markdown line have no parse tree, so the scan tracks their
# backticks itself.
#
# The history list is short on purpose: each entry reads as history in nearly
# every use. The exception is "used to" before a verb, as in "used to report
# it". The gate flags it, and rewriting the sentence is the fix. "moved",
# "retired" and "replaced" stay off the list, because the docs also use them
# for what a command does to a task.
#
# An all-caps word passes when it is an acronym in ALLOWED, a constant a Typst
# file declares with `let`, or a file name such as README.md.
set -eo pipefail

# shellcheck source=.just/scripts/_doc-blocks-lib.sh
source .just/scripts/_doc-blocks-lib.sh

ALLOWED="ADR AI ANSI API ASCII ATX CGO CI CLI CPU CQRS CRAP CSS CSV DB DOM DOT
DRY DSL ETA GB GIO GPL GUI HTML ICE ICU ID IEC ISBN ISC ISO JS JSON LLVM MB MCP
MIT OG OS OSC PDF PR PTY README RFC RICE RRULE SEO SGR SHA SQL SSG SVG SWHID
TO-BE TOML TSV TTY TUI UI URI URL UTC UUID VCS WIP WSJF XDG YAML"
HISTORY_WORDS="now anymore formerly"
HISTORY_PHRASES="used to|no longer"

# One awk pass over the rows on stdin: file, line, kind (prose or comment) and
# text, tab separated. The consts argument lists the constants the Typst files
# declare. Prints "file:line: message" per hit; a line of `-` names a schema
# key in the file column and prints "file: message".
scan() { # <consts>
    awk -F'\t' -v allowed="$ALLOWED $1" -v words="$HISTORY_WORDS" \
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
        function hit(msg) {
            if (line == "-") printf "%s: %s\n", file, msg
            else printf "%s:%d: %s\n", file, line, msg
        }
        function count(s, re,    k) {
            k = 0
            while (match(s, re)) { k++; s = substr(s, RSTART + RLENGTH - 1) }
            return k
        }
        {
            file = $1; line = $2; kind = $3; text = $4
            # An unclosed quote or inline span ends at a blank line, at a gap
            # between rows, or where comment rows meet prose rows. A comment
            # has no fenced blocks, so a stray fence ends there too.
            if (file != pfile || text !~ /[^ \t]/ || kind != pkind \
                || (line != "-" && line != pline + 1)) {
                quoted = 0; tail = ""
                if (raw == 1 || kind == "comment" || file != pfile) raw = 0
            }
            pfile = file; pkind = kind; pline = line
            v = visible(text)
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
    '
}

# The constants the given Typst files declare with `let`.
consts_of() { # <file>...
    cat "$@" | grep -oE '(^|[^A-Za-z0-9_-])let +[A-Z][A-Z0-9_]*' |
        sed -E 's/.*let +//' | sort -u | tr '\n' ' ' || true
}

# The rows of each file by kind. A Typst file comes from the extractor. A
# Justfile or YAML file yields its full-line `#` comments, and every other line
# as an empty row that ends an open quote. Any other file is Markdown, read as
# written.
rows_for() { # <file>...
    local f
    for f in "$@"; do
        case "$f" in
        *.typ) doc_extract "$f" | awk -F'\t' '$3 == "prose" || $3 == "comment"' ;;
        Justfile | *.just | *.yaml | *.yml)
            awk -v f="$f" '{
                gsub(/\t/, " ")
                t = ""
                if ($0 ~ /^[ \t]*#/) { t = $0; sub(/^[ \t]*#+/, "", t) }
                print f "\t" FNR "\tcomment\t" t
            }' "$f" ;;
        *) awk -v f="$f" '{ gsub(/\t/, " "); print f "\t" FNR "\tprose\t" $0 }' "$f" ;;
        esac
    done
}

# Every `description` of a JSON schema ($1, labelled as $2) as rows for `scan`.
# Each row carries the file and the key path in its file column and `-` as its
# line, so a hit names the key. The walk visits the places a schema nests
# another schema and skips `default` and `examples`, which hold data.
schema_text() {
    jq -r --arg f "$2" '
        def descs($p):
            if type != "object" then empty else
                (if (.description | type) == "string" then
                    {key: (if ($p | length) == 0 then "(root)" else $p | join(".") end),
                     text: .description}
                 else empty end),
                (("properties", "$defs", "definitions", "patternProperties") as $m
                 | (.[$m] // empty | select(type == "object") | to_entries[]
                    | .key as $k | .value | descs($p + [$m, $k]))),
                (("items", "additionalProperties", "not", "if", "then", "else") as $m
                 | (.[$m] // empty | descs($p + [$m]))),
                (("oneOf", "anyOf", "allOf") as $m
                 | (.[$m] // empty | select(type == "array") | to_entries[]
                    | .key as $i | .value | descs($p + [$m, ($i | tostring)])))
            end;
        descs([]) | "\($f):\(.key)" as $file
        | .text | split("\n")[] | [$file, "-", "prose", gsub("\t"; " ")] | join("\t")' "$1"
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
#let fence = "```"
After a string holding a fence, now is still prose.
EOF
    cat >"$dir/g.yaml" <<'EOF'
key: now used to be fine
# It is now SHOUTED here.
  # A `now` span and "no longer" quote pass.
EOF
    cat >"$dir/s.json" <<'EOF'
{"description": "Root now.",
 "properties": {
   "description": {"type": "object", "description": "A key named description."},
   "k": {"description": "A `now` span passes.", "default": {"description": "now"}}},
 "$defs": {"A": {"description": "Fine.", "oneOf": [
   {"description": "It used to be SHOUTED."}]}}}
EOF
    out=$({
        rows_for "$dir/f.typ" "$dir/g.yaml"
        schema_text "$dir/s.json" s.json
    } | scan "$(consts_of "$dir/f.typ")" | sed "s|^$dir/||")
    rm -rf "$dir"
    want='f.typ:3: history word "used to"
f.typ:4: history word "now"
f.typ:4: all-caps "SHOUTED"
f.typ:9: history word "now"
f.typ:9: history word "now"
f.typ:9: history word "no longer"
f.typ:11: history word "now"
g.yaml:2: history word "now"
g.yaml:2: all-caps "SHOUTED"
s.json:(root): history word "now"
s.json:$defs.A.oneOf.0: history word "used to"
s.json:$defs.A.oneOf.0: all-caps "SHOUTED"'
    if [ "$out" != "$want" ]; then
        printf 'check-doc-prose: the control fixture gave\n%s\nexpected\n%s\n' \
            "$out" "$want" >&2
        exit 1
    fi
}

control

doc_sources_load
typst_sources=()
for f in "${DOC_SOURCES[@]}" www/templates/*.typ; do
    # tola.typ is tola's own generated helper, never edited here.
    [ "$(basename "$f")" = "tola.typ" ] || typst_sources+=("$f")
done
shopt -s nullglob
other_sources=(skills/mindtape/references/*.md skills/mindtape/evals/README.md
    Justfile .just/*.just .mmz/config.yaml outdatty.yaml .linecop.yaml)

hits=$({
    rows_for "${typst_sources[@]}" "${other_sources[@]}"
    schema_text schema/config.schema.json schema/config.schema.json
} | scan "$(consts_of "${typst_sources[@]}")")
if [ -n "$hits" ]; then
    sed 's/^/check-doc-prose: /' <<<"$hits" >&2
    echo "check-doc-prose: rewrite each line plainly; the rules are in docs/contributing/documentation.md" >&2
    exit 1
fi
