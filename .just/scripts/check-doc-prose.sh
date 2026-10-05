#!/usr/bin/env bash
# .just/scripts/check-doc-prose.sh — the body of the `check-doc-prose` recipe.
# It stops doc prose, config comments and config schema descriptions from
# telling history or shouting in all-caps words.
#
# It scans the hand-written doc sources, prose and comments alike, and prints
# file, line and word for each hit. The Typst sources come through the shared
# extractor in _doc-blocks-lib.sh, which yields each line of rendered text and
# each comment line apart from raw spans, fenced blocks and code. Markdown
# sources have no Typst parse tree, so their lines are read as written.
#
# It also scans the full-line `#` comments of every shell script, YAML file,
# Justfile and `.just` module that git lists, tracked or not yet added, so a
# new script is read the day it lands. Code lines, shebangs, shellcheck
# directives and heredoc bodies in those files are not prose and are skipped.
# EXCLUDE names a file to skip, with its reason.
#
# It reads every `description` out of schema/config.schema.json too, because those texts reach the docs site and
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
# file declares with `let` or a shell file assigns, or a file name such as
# README.md.
set -eo pipefail

# shellcheck source=.just/scripts/_doc-blocks-lib.sh
source .just/scripts/_doc-blocks-lib.sh

ALLOWED="ADR AI ANSI API ASCII ATX CC CGO CI CLI CNAME CPU CQRS CRAP CSS CSV CWD
DB DOM DOT DRY DSL ERR ETA ETXTBSY FTL GB GIO GPL GUI HEAD HTML ICE ICU ID IEC
ISBN ISC ISO JS JSON LLVM MB MCP MIT OG OS OSC PATH PDF PR PTY README RFC RICE
RRULE RUSTFLAGS SEO SGR SHA SQL SSG STDERR STDIN STDOUT SVG SWHID TAB TO-BE TOML
TSV TTY TUI UI UNC URI URL UTC UUID VCS WIP WSJF XDG YAML"
HISTORY_WORDS="now anymore formerly"
HISTORY_PHRASES="used to|no longer"

# One awk pass over the rows on stdin, in check-doc-prose.awk. The consts
# argument lists the constants the scanned files declare.
scan() { # <consts>
    awk -F'\t' -v allowed="$ALLOWED $1" -v words="$HISTORY_WORDS" \
        -v phrases="$HISTORY_PHRASES" -f .just/scripts/check-doc-prose.awk
}

# The constants the given files declare: `let NAME` in Typst and `NAME=` at the
# start of a shell line, possibly after `local`.
consts_of() { # <file>...
    {
        cat "$@" | grep -oE '(^|[^A-Za-z0-9_-])let +[A-Z][A-Z0-9_]*' | sed -E 's/.*let +//'
        cat "$@" | grep -oE '^[ \t]*(local +|declare +-[A-Za-z]+ +)?[A-Z][A-Z0-9_]*=' | sed -E 's/^[ \t]*(local +|declare +-[A-Za-z]+ +)?//; s/=$//'
    } | sort -u | tr '\n' ' ' || true
}

# The rows of each file by kind. A Typst file comes from the extractor. A
# shell script, Justfile or YAML file yields its full-line `#` comments, and
# every other line as an empty row that ends an open quote. A shell script
# loses its shebang, its shellcheck directives and its heredoc bodies. Any
# other file is Markdown, read as written.
rows_for() { # <file>...
    local f
    for f in "$@"; do
        case "$f" in
        *.typ) doc_extract "$f" | awk -F'\t' '$3 == "prose" || $3 == "comment"' ;;
        *.sh)
            # A heredoc body is data, so its `#` lines are not comments.
            awk -v f="$f" '
                function row(t) { print f "\t" FNR "\tcomment\t" t }
                heredoc != "" {
                    l = $0
                    if (dash) sub(/^\t+/, "", l)
                    if (l == heredoc) heredoc = ""
                    row(""); next
                }
                $0 ~ /^[ \t]*#/ {
                    if (FNR == 1 && $0 ~ /^#!/ || $0 ~ /^[ \t]*#[ \t]*shellcheck /) { row(""); next }
                    gsub(/\t/, " "); sub(/^[ \t]*#+/, ""); row($0); next
                }
                {
                    if (match($0, /<<-?[ \t]*["\047]?[A-Za-z_][A-Za-z0-9_]*/)) {
                        d = substr($0, RSTART, RLENGTH)
                        dash = (d ~ /^<<-/)
                        sub(/^<<-?[ \t]*["\047]?/, "", d)
                        heredoc = d
                    }
                    row("")
                }' "$f" ;;
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
    cat >"$dir/h.sh" <<'EOF'
#!/usr/bin/env bash
# It is now a SHOUTED script comment.
# shellcheck disable=SC2016 # now is a directive
cat <<'TXT'
# now and SHOUTED text inside a heredoc is data
TXT
echo "# now in a string" # now after code is not a full-line comment
  # no longer true, in an indented comment
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
        rows_for "$dir/f.typ" "$dir/g.yaml" "$dir/h.sh"
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
h.sh:2: history word "now"
h.sh:2: all-caps "SHOUTED"
h.sh:8: history word "no longer"
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
md_sources=(skills/mindtape/references/*.md skills/mindtape/evals/README.md)

# The comment sources are every file git lists, tracked or not yet added, of
# the kinds below, so a new script is read the day it lands. EXCLUDE names the
# files to skip, each with its reason.
EXCLUDE=()
comment_sources=()
while IFS= read -r f; do
    skip=0
    for e in "${EXCLUDE[@]}"; do
        [ "${e%%|*}" = "$f" ] && skip=1
    done
    [ "$skip" -eq 1 ] || comment_sources+=("$f")
done < <(git ls-files --cached --others --exclude-standard -- \
    '*.sh' '*.just' '*.yaml' '*.yml' Justfile '*/Justfile')

hits=$({
    rows_for "${typst_sources[@]}" "${md_sources[@]}" "${comment_sources[@]}"
    schema_text schema/config.schema.json schema/config.schema.json
} | scan "$(consts_of "${typst_sources[@]}" "${comment_sources[@]}")")
if [ -n "$hits" ]; then
    sed 's/^/check-doc-prose: /' <<<"$hits" >&2
    echo "check-doc-prose: rewrite each line plainly; the rules are in docs/contributing/documentation.md" >&2
    exit 1
fi
