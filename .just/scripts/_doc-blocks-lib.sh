# .just/scripts/_doc-blocks-lib.sh — the scanner `just check-doc-blocks` runs,
# sourced by check-doc-blocks.sh and by its control. Runs from the repo root.
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
            echo "check-doc-blocks: $f has no hand-typed block left; drop it from PENDING in .just/scripts/check-doc-blocks.sh" >&2
            fail=1
        fi
    done
    return $fail
}
