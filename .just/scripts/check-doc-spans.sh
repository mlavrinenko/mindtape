#!/usr/bin/env bash
# .just/scripts/check-doc-spans.sh — the body of the `check-doc-spans` recipe.
# Runs from the repo root, like the recipe that invokes it.
#
# Stops prose that names an `mt` subcommand, flag or config key that does not
# exist. Inline code in the docs names these by hand, so a rename in
# crates/mindtape-cli or in the config structs leaves the old name in the
# prose. This gate fails on it, naming file, line and the closest valid name.
#
# It reads inline code spans, as backtick markup and as `raw("...")`, in the
# sources the manifest in _doc-blocks-lib.sh names. Fenced blocks and comments
# are not spans. Two checks read each span:
#
#   - An `mt` command line: the span is split at `|`, `&&` and `;`, and each
#     piece that starts with `mt`, after a `$ ` prompt or `VAR=value` words, goes
#     to `mt _cli --check-argv`. That is the parser that runs the command, so
#     the gate holds no copy of its grammar. The status verbs the project and
#     the examples declare are passed along, because each project names its own.
#   - check-doc-spans.jq reads the config shapes. `[a.b]` or `[[a.b]]`,
#     optionally followed by a key: the path exists in
#     schema/config.schema.json, and `[[...]]` is used exactly for an array of
#     tables. `a.b.c`: the key path exists in the schema.
#
# A span is config-shaped only when its first segment is a top-level schema
# key. That leaves Cargo tables and just attributes alone. A dotted span that
# is a tracked file's name, such as `typst.toml`, is a file name.
#
# A control runs first. It checks a synthetic page holding each known-bad
# shape beside the good ones and asserts the exact findings, because a scanner
# that matched nothing also prints no findings. Its bad flags are ones no
# command will ever take, so a new flag on `mt ls` cannot fail it.
set -eo pipefail
# shellcheck source=.just/scripts/_doc-blocks-lib.sh
source .just/scripts/_doc-blocks-lib.sh
# `--tests`: docs/contributing/gates.md, "One mt binary, one feature set".
cargo build -p mindtape-cli --tests -q
mt=./target/debug/mt
tmp=$(mktemp -d -p target)
trap 'rm -rf "$tmp"' EXIT

# `mt <status>` runs `mt flip <status>`, and each project declares its own
# statuses. The docs show this project and the examples, so the verb set is
# the union of their legends. The empty homes drop any user config layer.
for d in . examples/*/; do
    [ -d "$d/.mindtape" ] || continue
    MINDTAPE_CONFIG_HOME='' MINDTAPE_DATA_HOME='' "$mt" -w "$d" config show --json
done | jq -s -r '[.[].tasks.status[]?.value] | unique | join(",")' >"$tmp/statuses"
git ls-files | sed 's|.*/||' | sort -u | jq -R . | jq -s . >"$tmp/files.json"

# Prints `<file>\t<line>\t<span>` for every inline span the extractor finds
# in the given sources. It reads the parse tree, so a fence inside a string
# hides nothing and a span may run onto the next line.
spans() {
    doc_extract "$@" | awk -F'\t' '$3 == "span" { print $1 "\t" $2 "\t" $4 }'
}

# Prints `<file>\t<line>\t<span>\t<command line>` for every `mt` command
# line in a span: each piece between the words `|`, `||`, `&&` and `;`
# that starts with `mt`, once a leading `$ ` and `VAR=value` words are dropped. The program writes the single quote as
# \u0027 because it sits inside a single-quoted shell string.
# shellcheck disable=SC2016
ARGV_JQ='split("\t") as [$file, $line, $span]
    | $span
    | [scan("(?:\u0027[^\u0027]*\u0027|\"[^\"]*\"|[^\\s\u0027\"])+")]
    | reduce .[] as $w ([[]]; if $w | IN("|", "||", "&&", ";") then . + [[]] else .[-1] += [$w] end)
    | .[]
    | if .[0] == "$" then .[1:] else . end
    | until((.[0] // "") | test("^[A-Za-z_][A-Za-z0-9_]*=") | not; .[1:])
    | select(.[0] == "mt" and length > 1)
    | [$file, $line, $span, join(" ")] | @tsv'

# One finding per `mt` command line that `mt _cli --check-argv` refuses.
argv_findings() {
    local statuses file line span cmd msg
    statuses=$(<"$tmp/statuses")
    spans "$@" | jq -R -r "$ARGV_JQ" |
        while IFS=$'\t' read -r file line span cmd; do
            if ! msg=$("$mt" _cli --statuses "$statuses" --check-argv "$cmd" 2>&1); then
                echo "check-doc-spans: $file:$line: \`$span\`: ${msg#error: }"
            fi
        done
}

check() {
    argv_findings "$@"
    spans "$@" | jq -R -r -f .just/scripts/check-doc-spans.jq \
        --slurpfile schema schema/config.schema.json \
        --slurpfile files "$tmp/files.json"
}

# The finding for a span that quotes `mt` output by hand.
# shellcheck disable=SC2016 # the backticks are literal Typst, not a command substitution
QUOTED='quotes `mt` output by hand; render the line with capture-line(<slug>, <regex>) from www/utils/ui.typ'

control() {
    local page=$tmp/control.typ
    cat >"$page" <<'EOF'
Good: `mt ls 'status != "done"' --sort priority:desc -n 5`, `mt done x -m "why"`,
`mt _cli --json`, `mt -w examples/minimal ls --json | mt import --dry-run`,
`mt <cmd> --help`, `mt config show --origin`, `mt ls -in20`, `mt @name ls --json`,
`cd x && mt ls`, `MINDTAPE_NOW=2026-01-01 mt ls --json`, `$ mt ls`, `mt ls --sort`.
Good: `[[tasks.roots]]`, `[tasks.priority]`, `[views.<name>]`, `[tasks] roots`,
`[ls.columns.assignee] when = 'true'`, `[[typst.local-packages]] path`,
`tasks.estimate.units`, `[workspace.package] version`, `[overlay]`, `typst.toml`.
Bad: `mt lss`, `mt ls --jsn`, `mt ls --no-such-flag`, `mt config shw`, `mt @name lsz`.
Bad: `cd x && mt lsj`, `MINDTAPE_NOW=2026-01-01 mt ls --jsn`, `$ mt lsq`, `mt ls | mt lsp`.
Bad: `[tasks.priorty]`, `tasks.estimate.unit`, `[tasks.roots]`, `[[tasks.priority]]`,
`[ls] hyperlinkz`, `[tasks.estimate.units]`. #raw("mt lsr")
Good: `error:`, `hint:`, `error codes`. Bad: `hint: file it with mt add -t dev`, #raw("error: x: y is bad")
// `mt lsc` in a comment line
```
mt lsf in a fenced block
```
examples: ```sh
mt lsg in a block opened mid-line
```, then `mt lsh` after it.
#let fence = "```"
After a string holding a fence: `mt lsi` is still a span.
EOF
    local want got
    want=$(sort <<'EOF'
`[[tasks.priority]]`: [[tasks.priority]] names a table; write [tasks.priority]
`[ls] hyperlinkz`: ls has no key hyperlinkz; closest: hyperlinks
`error: x: y is bad`: QUOTED
`hint: file it with mt add -t dev`: QUOTED
`[tasks.estimate.units]`: [tasks.estimate.units] names a value, not a table
`[tasks.priorty]`: tasks has no key priorty; closest: priority
`[tasks.roots]`: [tasks.roots] names an array of tables; write [[tasks.roots]]
`mt config shw`: mt config has no subcommand shw; closest: show
`mt ls --jsn`: mt ls has no flag --jsn; closest: --json
`mt ls --no-such-flag`: mt ls has no flag --no-such-flag
`cd x && mt lsj`: no subcommand lsj; closest: ls
`MINDTAPE_NOW=2026-01-01 mt ls --jsn`: mt ls has no flag --jsn; closest: --json
`$ mt lsq`: no subcommand lsq; closest: ls
`mt ls | mt lsp`: no subcommand lsp; closest: ls
`mt lsh`: no subcommand lsh; closest: ls
`mt lsi`: no subcommand lsi; closest: ls
`mt lsr`: no subcommand lsr; closest: ls
`mt @name lsz`: no subcommand lsz; closest: ls
`mt lss`: no subcommand lss; closest: ls
`tasks.estimate.unit`: tasks.estimate has no key unit; closest: units
EOF
    )
    want=${want//QUOTED/$QUOTED}
    got=$(check "$page" | sed "s|^check-doc-spans: $page:[0-9]*: ||" | sort)
    if [ "$got" != "$want" ]; then
        echo "check-doc-spans: control failed; findings differ from the expected set:" >&2
        diff <(echo "$want") <(echo "$got") >&2 || true
        exit 1
    fi
}

control

doc_sources_load
findings=$(check "${DOC_SOURCES[@]}")
if [ -n "$findings" ]; then
    echo "$findings" >&2
    exit 1
fi
