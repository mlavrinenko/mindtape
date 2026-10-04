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
# are not spans. check-doc-spans.jq checks three shapes:
#
#   - `mt <word> ...`: the word is a subcommand from `mt _cli --json` or a
#     status verb. Every flag must exist on that command or be global. A quoted
#     word is an argument, so a filter is never read as flags. A leading
#     `@name` right after `mt` is worktree sugar and is skipped.
#   - `[a.b]` or `[[a.b]]`, optionally followed by a key: the path exists in
#     schema/config.schema.json, and `[[...]]` is used exactly for an array of
#     tables.
#   - `a.b.c`: the key path exists in the schema.
#
# A span is config-shaped only when its first segment is a top-level schema
# key. That leaves Cargo tables and just attributes alone. A dotted span that
# is a tracked file's name, such as `typst.toml`, is a file name.
#
# A control runs first. It checks a synthetic page holding each known-bad
# shape beside the good ones and asserts the exact findings, because a scanner
# that matched nothing also prints no findings.
set -eo pipefail
# shellcheck source=.just/scripts/_doc-blocks-lib.sh
source .just/scripts/_doc-blocks-lib.sh
# `--tests`: docs/contributing/gates.md, "One mt binary, one feature set".
cargo build -p mindtape-cli --tests -q
mt=./target/debug/mt
tmp=$(mktemp -d -p target)
trap 'rm -rf "$tmp"' EXIT

# `mt _cli --json` leaves out hidden commands, and the dump command is one of
# them. The call below shows that its name and flag exist, so the gate adds
# the command from that call.
dump=(_cli --json)
"$mt" "${dump[@]}" | jq --arg name "${dump[0]}" --arg flag "${dump[1]#--}" \
    '.commands += [{name: $name, subcommands: [],
        args: [{long: $flag, short: null, kind: "flag"}]}]' >"$tmp/cli.json"

# `mt <status>` runs `mt flip <status>`, and each project declares its own
# statuses. The docs show this project and the examples, so the verb set is
# the union of their legends. The empty homes drop any user config layer.
for d in . examples/*/; do
    [ -d "$d/.mindtape" ] || continue
    MINDTAPE_CONFIG_HOME= MINDTAPE_DATA_HOME= "$mt" -w "$d" config show --json
done | jq -s '[.[].tasks.status[]?.value] | unique' >"$tmp/statuses.json"
git ls-files | sed 's|.*/||' | sort -u | jq -R . | jq -s . >"$tmp/files.json"

# Prints `<file>\t<line>\t<span>` for every inline span the extractor finds
# in the given sources. It reads the parse tree, so a fence inside a string
# hides nothing and a span may run onto the next line.
spans() {
    doc_extract "$@" | awk -F'\t' '$3 == "span" { print $1 "\t" $2 "\t" $4 }'
}

check() {
    spans "$@" | jq -R -r -f .just/scripts/check-doc-spans.jq \
        --slurpfile cli "$tmp/cli.json" \
        --slurpfile schema schema/config.schema.json \
        --slurpfile statuses "$tmp/statuses.json" \
        --slurpfile files "$tmp/files.json"
}

control() {
    local page=$tmp/control.typ
    cat >"$page" <<'EOF'
Good: `mt ls 'status != "done"' --sort priority:desc -n 5`, `mt done x -m "why"`,
`mt _cli --json`, `mt -w examples/minimal ls --json | mt import --dry-run`,
`mt <cmd> --help`, `mt config show --origin`, `mt ls -in20`, `mt @name ls --json`.
Good: `[[tasks.roots]]`, `[tasks.priority]`, `[views.<name>]`, `[tasks] roots`,
`[ls.columns.assignee] when = 'true'`, `[[typst.local-packages]] path`,
`tasks.estimate.units`, `[workspace.package] version`, `[overlay]`, `typst.toml`.
Bad: `mt lss`, `mt ls --jsn`, `mt ls -x`, `mt config shw`, `mt @name lsz`.
Bad: `[tasks.priorty]`, `tasks.estimate.unit`, `[tasks.roots]`, `[[tasks.priority]]`,
`[ls] hyperlinkz`, `[tasks.estimate.units]`. #raw("mt lsr")
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
`[tasks.estimate.units]`: [tasks.estimate.units] names a value, not a table
`[tasks.priorty]`: tasks has no key priorty; closest: priority
`[tasks.roots]`: [tasks.roots] names an array of tables; write [[tasks.roots]]
`mt config shw`: mt config has no subcommand shw; closest: show
`mt ls --jsn`: mt ls has no flag --jsn; closest: --json
`mt ls -x`: mt ls has no flag -x; valid: -i, -n, -v, -w
`mt lsh`: no subcommand lsh; closest: ls
`mt lsi`: no subcommand lsi; closest: ls
`mt lsr`: no subcommand lsr; closest: ls
`mt @name lsz`: no subcommand lsz; closest: ls
`mt lss`: no subcommand lss; closest: ls
`tasks.estimate.unit`: tasks.estimate has no key unit; closest: units
EOF
    )
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
