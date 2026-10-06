# .just/scripts/_doc-blocks-lib.sh — the doc sources' shared reader. It holds
# the scanner `just check-doc-blocks` runs, the extractor that check-doc-spans,
# check-doc-recipes and check-doc-prose read, and the source manifest. Sourced
# by those gates and by the blocks control. Runs from the repo root.
#
# The scanner reads Typst's own parse tree (`typstyle --ast`, the formatter the
# dev shell already pins) instead of matching text, so a fence inside a
# comment, a string or another raw span is never mistaken for a block.
# doc_blocks_scan prints one TSV row per finding: file, line, kind, detail.
# Kind `block` is an unmarked hand-typed block; kind `marker` is a broken
# `// docs:hand-typed` marker.

# The typstyle dump as JSON: {"f": file, "n": tree}. Inner nodes carry `c`
# (children), leaves carry `t` (source text), and every node carries `l`, the
# line it starts on, counted from the newlines in the leaves before it.
doc_blocks_ast() { # <file>
    typstyle --ast "$1" | LC_ALL=C awk -v file="$1" '
        # Rust debug escapes to JSON escapes; counts each \n into `nl`.
        function jsonify(s,   out, i, c, hex) {
            out = ""; nl = 0
            while ((i = index(s, "\\")) > 0) {
                out = out substr(s, 1, i - 1)
                c = substr(s, i + 1, 1)
                if (c == "u" && match(substr(s, i + 2), /^\{[0-9a-fA-F]+\}/)) {
                    hex = substr(s, i + 3, RLENGTH - 2)
                    out = out (length(hex) <= 4 ? "\\u" substr("0000" hex, length(hex) + 1) : "\\ufffd")
                    s = substr(s, i + 2 + RLENGTH)
                    continue
                }
                if (c == "n") nl++
                out = out (c == "0" ? "\\u0000" : "\\" c)
                s = substr(s, i + 2)
            }
            return out s
        }
        function sep() { if (!first) printf ","; first = 0 }
        BEGIN { line = 1; first = 1; printf "{\"f\":\"%s\",\"n\":", file }
        # The formatted source follows the tree; read it to the end so
        # typstyle never dies on a closed pipe.
        done { next }
        /^\]$/ { print "]}}"; done = 1; next }
        /^ *\],?$/ { printf "]}"; first = 0; next }
        /^ *[A-Za-z]+: [0-9]+ \[$/ {
            sep(); sub(/^ */, ""); printf "{\"k\":\"%s\",\"l\":%d,\"c\":[", substr($0, 1, index($0, ":") - 1), line
            first = 1; next
        }
        /^ *[A-Za-z]+: [0-9]+,?$/ {
            sep(); sub(/^ */, ""); printf "{\"k\":\"%s\",\"l\":%d,\"c\":[]}", substr($0, 1, index($0, ":") - 1), line
            next
        }
        /^ *[A-Za-z]+: ".*",?$/ {
            sep(); sub(/^ */, ""); kind = substr($0, 1, index($0, ":") - 1)
            text = $0; sub(/^[A-Za-z]+: "/, "", text); sub(/",?$/, "", text)
            printf "{\"k\":\"%s\",\"l\":%d,\"t\":\"%s\"}", kind, line, jsonify(text)
            line += nl; next
        }
        { printf "check-doc-blocks: unreadable typstyle --ast line in %s: %s\n", file, $0 > "/dev/stderr"; exit 2 }
        END { if (!done) exit 2 }
    '
}

# The source manifest: the files every doc gate reads, one glob per line.
# docs/src/lib.typ is a helper module and joins the list; the extractor yields
# its comments apart from its prose, so no gate mistakes a comment for a page.
DOC_SOURCE_GLOBS=(
    'docs/src/**/*.typ'
    'www/content/**/*.typ'
    'www/utils/*.typ'
)

# doc_sources_load fills DOC_SOURCES from the manifest. A glob that matches no
# file fails, because a gate that reads nothing reports a clean tree.
doc_sources_load() {
    local g f found
    DOC_SOURCES=()
    shopt -s globstar nullglob
    for g in "${DOC_SOURCE_GLOBS[@]}"; do
        found=0
        for f in $g; do
            DOC_SOURCES+=("$f")
            found=1
        done
        if [ "$found" -eq 0 ]; then
            echo "doc sources: the glob $g matches no file" >&2
            return 2
        fi
    done
}

# The extractor. It reads the same tree and prints one TSV row per piece of a
# Typst source: file, line, kind, text. The text has no tab or newline.
#   span     an inline raw: backticks (fewer than three) or `raw("...")`. A
#            backtick pair inside a string literal also counts, because
#            metadata strings such as a page summary name commands in them.
#   block    one line of a block raw: three or more backticks, or
#            `raw("...", block: true)`. The language tag is not text.
#   comment  one line of a `//` or `/* */` comment, delimiters removed.
#   prose    one source line of rendered text: what the markup says outside
#            raw, comments and code. Each raw or code expression leaves one
#            space, and a double quote stays a `"`.
# A fence written inside a string is code, so it opens no block.
# shellcheck disable=SC2016
DOC_EXTRACT_JQ='
def markup_container: IN("Strong","Emph","Heading","ListItem","EnumItem","TermItem");
def rawargs: [.c[] | select(.k == "Args") | .c[]
  | select(.k | IN("Space","Comma","LeftParen","RightParen","LineComment","BlockComment") | not)];
def rawcall: .k == "FuncCall" and .c[0].k == "Ident" and .c[0].t == "raw";
def strlit: .t | try fromjson catch .[1:-1];
def rawcall_events:
  rawargs as $a
  | if ($a[0].k // "") == "Str" then
      ($a[0] | strlit) as $text
      | if any($a[]; .k == "Named" and .c[0].t == "block" and .c[-1].t == "true")
        then $text | split("\n") | to_entries[] | {l: ($a[0].l + .key), kind: "block", t: .value}
        else {l: $a[0].l, kind: "span", t: $text} end
    else empty end;
def ev($m):
  if .k == "Raw" then
    (if (.c[0].t | length) >= 3
     then .c[] | select(.k == "Text") | {l, kind: "block", t}
     else {l, kind: "span",
           t: ([.c[] | select(.k == "Text" or .k == "RawTrimmed") | if .k == "Text" then .t else " " end] | join(""))}
     end),
    {l, kind: "prose", t: " "}
  elif .k == "LineComment" then {l, kind: "comment", t: (.t | sub("^//+"; ""))}
  elif .k == "BlockComment" then {l, kind: "comment", t: (.t | sub("^/\\*+"; "") | sub("\\*+/$"; ""))}
  elif .k == "Markup" then .c[] | ev("m")
  elif .c then
    if rawcall then rawcall_events, {l, kind: "prose", t: " "}
    elif $m == "m" and (.k | markup_container) then .c[] | ev("m")
    else .c[] | ev("c") end
  elif .k | IN("LeftBracket","RightBracket") then {l, kind: "prose", t: " "}
  elif .k == "Str" then
    . as $s | strlit | match("`([^`\n]+)`"; "g") | {l: $s.l, kind: "span", t: .captures[0].string}
  elif $m == "m" then
    if .k | IN("Text","Space","SmartQuote") then {l, kind: "prose", t}
    elif .k | IN("Hash","Escape","Shorthand","Link","Label","Ref","RefMarker","Linebreak") then {l, kind: "prose", t: " "}
    else empty end
  else empty end;
def rows:
  reduce .[] as $e ({out: [], cur: null};
    if $e.kind == "span" or $e.kind == "block" then .out += [$e]
    else
      reduce range(0; $e.t | split("\n") | length) as $i (.;
        ($e.t | split("\n")[$i]) as $seg | ($e.l + $i) as $ln
        | if .cur != null and .cur.l == $ln and .cur.kind == $e.kind then .cur.t += $seg
          else (if .cur != null then .out += [.cur] else . end) | .cur = {l: $ln, kind: $e.kind, t: $seg} end)
    end)
  | (.out + [.cur // empty]) | sort_by(.l);
[inputs] | .[] | .f as $f
| [.n | ev("m")] | rows | .[]
| select(.t | test("\\S"))
| [$f, .l, .kind, (.t | gsub("[\t\r]"; " "))] | join("\t")
'

doc_extract() { # <file>...
    local f
    for f in "$@"; do
        doc_blocks_ast "$f" || {
            echo "doc extract: typstyle --ast failed on $f" >&2
            return 2
        }
    done | jq -nr "$DOC_EXTRACT_JQ"
}

# The tree rules. A block is hand-typed when a three-backtick raw spans a line
# break, or when a block-producing call gets text written in place: a string or
# raw literal outside any call's arguments, or a `let` bound to one. A call is
# block-producing when it is `raw(..., block:)` with anything but `false`, or a
# helper: a function whose own parameter reaches such a call, followed through
# `let`s, method targets and plain calls. Loaders (`read`, `json`, ...) and
# method arguments end that path, so `terminal(read(...))` and
# `framework-call(fw, "ice")` pass. Helpers are collected across every scanned
# file first, because pages call the ones www/utils defines. A `let` bound to a
# literal is followed within its own file only.
# shellcheck disable=SC2016
DOC_BLOCKS_JQ='
def trivia: IN("Space","LineComment","BlockComment","Comma","LeftParen","RightParen","Colon","Semicolon","Hash","Shebang");
def nodes: recurse(.c[]?);
def fix(f): def r: (f) as $n | if $n == . then . else $n | r end; r;
def args: .c[] | select(.k == "Args") | .c[];
def positional: [args | select((.k | trivia | not) and .k != "Named" and .k != "Spread")];
def named($n): args | select(.k == "Named" and .c[0].t == $n) | last(.c[] | select(.k | trivia | not));
def callee: .c[0] | if .k == "Ident" then .t elif .k == "FieldAccess" then .c[-1].t else null end;
def blockish: [named("block")] | length > 0 and (.[0] | .k == "Bool" and .t == "false" | not);
def lets: nodes | select(.k == "LetBinding") | [.c[] | select(.k | trivia | not)]
  | select(.[1].k == "Ident" and .[2].k == "Eq") | {name: .[1].t, init: .[-1]};
def lit($v):
  if .k == "Str" or .k == "Raw" then true
  elif .k == "Ident" then $v[.t] == true
  elif .k == "Args" or .k == "Closure" then false
  elif .k == "FieldAccess" then .c[0] | lit($v)
  else any(.c[]?; lit($v)) end;
def litvars($lets): {} | fix(. as $v | reduce $lets[] as $l ($v;
  if .[$l.name] then . elif ($l.init | lit($v)) then .[$l.name] = true else . end));
def flows:
  if .k == "Ident" then .t
  elif .k == "FieldAccess" then .c[0] | flows
  elif .k == "Closure" then empty
  elif .k == "FuncCall" then
    if .c[0].k == "FieldAccess" then .c[0].c[0] | flows
    elif (.c[0].t | IN("read","json","yaml","toml","csv","xml","cbor")) then empty
    else args | flows end
  else .c[]? | flows end;
def tainted($p; $lets): {($p): true} | fix(. as $t | reduce $lets[] as $l ($t;
  if .[$l.name] then . elif any($l.init | flows; $t[.]) then .[$l.name] = true else . end));
def textargs($h):
  if .c[0].k == "Ident" and .c[0].t == "raw" then (if blockish then positional[0:1][] else empty end)
  elif $h[callee // ""] then $h[callee] as $x | positional as $p
    | ($x.pos[] | $p[.] // empty), ($x.named[] as $n | named($n))
  else empty end;
def closures: nodes | select(.k == "LetBinding") | [.c[] | select(.k | trivia | not)]
  | if .[1].k == "Closure" then {name: first(.[1].c[] | select(.k == "Ident") | .t), cl: .[1]}
    elif .[1].k == "Ident" and .[-1].k == "Closure" then {name: .[1].t, cl: .[-1]}
    else empty end
  | ([.cl.c[] | select(.k == "Params") | .c[]]) as $ps
  | .pos = [$ps[] | select(.k == "Ident") | .t] | .named = [$ps[] | select(.k == "Named") | .c[0].t]
  | .lets = [.cl | lets];
def withs: lets | select(.init.k == "FuncCall" and .init.c[0].k == "FieldAccess"
  and .init.c[0].c[0].t == "raw" and .init.c[0].c[-1].t == "with" and (.init | blockish))
  | {(.name): {pos: [0], named: []}};
def helpers($files): [$files[].n | closures] as $cls
  | ([$files[].n | withs] | add // {})
  | fix(. as $h | reduce $cls[] as $c ($h;
      [$c.cl | nodes | select(.k == "FuncCall") | textargs($h)] as $ta
      | def reaches($p): tainted($p; $c.lets) as $t | any($ta[] | flows; $t[.]);
        {pos: [$c.pos | to_entries[] | select(.value as $p | reaches($p)) | .key],
         named: [$c.named[] | select(reaches(.))]} as $x
      | if ($x.pos + $x.named | length) > 0 then .[$c.name] = $x else . end));
[inputs] as $files | helpers($files) as $h
| $files[] | .f as $f | .n as $root | litvars([$root | lets]) as $v
| ([$root | nodes | select(.k == "LineComment" and (.t | test("^//\\s*docs:hand-typed")))]) as $marks
| [ ($root | nodes | select(.k == "Raw" and (.c[0].t | length) >= 3
        and any(.c[]; .k != "RawDelim" and (.t // "" | contains("\n")))) | {l, what: "a fenced block"}),
    ($root | nodes | select(.k == "FuncCall") | select(any(textargs($h); lit($v)))
        | {l, what: "the text of \(callee)(...)"}) ] as $blocks
| ($marks | map(select(.t | test("^// docs:hand-typed\\s+\\S"))) | map(.l)) as $ok
| ($blocks | unique_by(.l)[] | select(.l - 1 | IN($ok[]) | not)
    | [$f, .l, "block", .what]),
  ($marks[] | if (.t | test("^// docs:hand-typed\\s+\\S") | not)
      then [$f, .l, "marker", "needs the form // docs:hand-typed <reason>"]
      elif (.l + 1 | IN($blocks[].l) | not)
      then [$f, .l, "marker", "excuses no block on the next line"]
      else empty end)
| @tsv
'

doc_blocks_scan() { # <file>...
    local f
    for f in "$@"; do
        doc_blocks_ast "$f" || {
            echo "check-doc-blocks: typstyle --ast failed on $f" >&2
            return 2
        }
    done | jq -nr "$DOC_BLOCKS_JQ" | sort -t$'\t' -k1,1 -k2,2n
}

# The list rule over a scan: an unlisted file with a block fails, a listed file
# without one fails, and a broken marker fails anywhere. Prints why on stderr.
doc_blocks_judge() { # <scan-tsv-file> <pending-file>...
    local scan=$1 f fail=0
    shift
    declare -A pending=()
    for f in "$@"; do pending[$f]=1; done
    while IFS=$'\t' read -r f line kind what; do
        if [ "$kind" = marker ]; then
            echo "check-doc-blocks: $f:$line: docs:hand-typed $what" >&2
            fail=1
        elif [ -z "${pending[$f]}" ]; then
            echo "check-doc-blocks: $f:$line: $what is typed by hand; slice it from a checked fixture, read a capture, or mark it // docs:hand-typed <reason>" >&2
            fail=1
        fi
    done <"$scan"
    for f in "$@"; do
        if ! awk -F'\t' -v f="$f" '$1 == f && $3 == "block" { found = 1 } END { exit !found }' "$scan"; then
            echo "check-doc-blocks: $f has no hand-typed block left; drop it from the PENDING list in check-doc-blocks" >&2
            fail=1
        fi
    done
    return $fail
}
