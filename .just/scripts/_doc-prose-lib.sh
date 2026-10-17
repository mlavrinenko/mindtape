#!/usr/bin/env bash
# .just/scripts/_doc-prose-lib.sh — the word lists and readers `just
# check-doc-prose` scans with. Sourced by check-doc-prose.sh, which holds the
# control and the run. Runs from the repo root.
#
# `rows_for`, `schema_text`, `string_rows` and `cli_text` turn each kind of
# source into tab-separated rows (file, line, kind, text). `scan` hands the
# rows to check-doc-prose.awk, and `name_scan` finds a typed product name.

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
# argument lists the constants the scanned files declare, dash is the regex of
# the files that must hold no em dash, and dashonly the regex of the files that
# get that rule and no other.
scan() { # <consts> <dash regex> <dash-only regex>
    awk -F'\t' -v allowed="$ALLOWED" -v consts="$1" -v words="$HISTORY_WORDS" \
        -v phrases="$HISTORY_PHRASES" -v dash="$2" -v dashonly="$3" -f .just/scripts/check-doc-prose.awk
}

# The constants the given files assign: `NAME=` at the start of a shell line,
# possibly after `local`.
consts_of() { # <file>...
    {
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

# The string literals of a Typst source that hold an em dash, as rows for
# `scan`. The extractor yields no string, because code is not prose, but a
# metadata summary or a `render-slices` note puts its string on the page.
string_rows() { # <typst file>...
    local f
    for f in "$@"; do
        doc_blocks_ast "$f" | jq -r '
            def strlit: .t | try fromjson catch .[1:-1];
            .f as $f | [.n | recurse(.c[]?) | select(.k == "Str" and (strlit | contains("—")))][]
            | [$f, .l, "prose", (strlit | gsub("[\t\r\n]"; " "))] | join("\t")'
    done
}

# Every string of the `mt _cli --json` dump on stdin as rows for `scan`, with
# the command path and argument id in the file column and `-` as the line.
cli_text() {
    jq -r '
        def texts($p):
            if type == "string" then {key: ($p | join(".")), text: .}
            elif type == "object" then to_entries[] | .key as $k | .value | texts($p + [$k])
            elif type == "array" then .[] as $e
                | ($e | if type == "object" then (.name // .id // "") else "" end) as $n
                | $e | texts(if $n == "" then $p else $p + [$n] end)
            else empty end;
        texts([]) | "cli.json:\(.key)" as $file
        | .text | split("\n")[] | [$file, "-", "prose", gsub("\t"; " ")] | join("\t")'
}

# The product's name. A Typst source renders it as `#mindtape`, from the
# constant www/utils/site.typ reads out of the site title in www/tola.toml, so
# a typed name in either spelling is a hit wherever it sits: prose, a string or
# a comment. A Markdown source can't reach the constant, so there only the
# old spelling is a hit. Prints "file:line: message" per hit.
name_scan() { # <typst file>... -- <markdown file>...
    local typ=() md=()
    while [ "$#" -gt 0 ] && [ "$1" != "--" ]; do typ+=("$1"); shift; done
    [ "$#" -gt 0 ] && shift
    md=("$@")
    if [ "${#typ[@]}" -gt 0 ]; then
        grep -nHoE 'Mind[Tt]ape' "${typ[@]}" | sed -E 's/^([^:]+:[0-9]+):(.*)$/\1: typed name "\2", write #mindtape/'
    fi
    if [ "${#md[@]}" -gt 0 ]; then
        grep -nHoE 'MindTape' "${md[@]}" | sed -E 's/^([^:]+:[0-9]+):.*$/\1: spelling "MindTape", write Mindtape/'
    fi
    return 0
}
