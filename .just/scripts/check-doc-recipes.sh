#!/usr/bin/env bash
# .just/scripts/check-doc-recipes.sh — the body of the `check-doc-recipes`
# recipe, extracted under truth/tasks/extract-the-long-gate-recipe-bodies-into-
# scripts.typ. Every path below is repo-root-relative, because the recipe that
# invokes it runs at the repo root.
#
# The gate for what www/generated/just.typ cannot reach: a fenced code block
# cannot call a function, and typst's own "module does not contain" error at
# render time is the gate for everything else — a doc source that writes
# `#just.<name>()` already fails `just docs::md` loudly the moment `<name>`
# stops resolving (see truth/tasks/render-just-recipe-names-through-a-
# generated-typst-module.typ). What is left uncovered is a hand-typed
# mention that never went through the helper at all.
#
# Ordinary English "just" (an adverb, or a config value's own `#` shell
# comment mid-line) matches none of the three shapes below and is never
# touched — verified against every non-recipe "just" already in the docs tree.
#
# A control runs first, on a synthetic page, so an extractor that misreads the
# fences cannot pass by finding nothing.
set -eo pipefail
# The same recipe/alias names www/generated/just.typ's own producer
# reads — never a second source of truth for "is this a real recipe".
# Module recipes (truth/tasks/split-the-justfile-into-modules.typ) are
# walked recursively and resolvable two ways: the module's own name alone
# (a hand-typed "just docs ..." example's captured word is only ever the
# FIRST token after "just", so a module invocation resolves to its bare
# name here) and the fully-qualified "module::recipe" path.
names=$(just --dump --dump-format json | jq -r '
    def rec_names(prefix):
      (.recipes // {} | keys[] | prefix + .),
      ( (.modules // {}) | to_entries[] | . as $e |
          (prefix + $e.key),
          ($e.value | rec_names(prefix + $e.key + "::"))
      );
    rec_names(""), (.aliases | keys[])
')
names_file=$(mktemp)
trap 'rm -f "$names_file"' EXIT
printf '%s\n' "$names" >"$names_file"

# shellcheck source=.just/scripts/_doc-blocks-lib.sh
source .just/scripts/_doc-blocks-lib.sh
fail=0

# Prints findings for the given doc sources to stderr and sets `fail`. The
# extractor (_doc-blocks-lib.sh) has already split each source into spans,
# block lines and prose, so a fence written inside a string hides nothing and
# a `//` comment line is never a row. The three shapes of literal mention
# follow, and are matched independently on each row:
#
#   - a quoted `"just <name>"` substring in a span, a block line or prose: a
#     literal value, not a live reference (an example `.mindtape/config.toml`
#     flip hook's `run = "just check"`). It never has to convert, but it has
#     to resolve. A word starting with `-` is a just flag such as `--fmt`,
#     which has no recipe to resolve.
#   - `just <name>` opening a line inside a block: a shell-command example. A
#     block cannot call the helper, so this is legal as prose, but the name
#     still has to resolve. A flag such as `just --list` names no recipe.
#   - a span opening with `just`: the hand-typed mention the generated module
#     exists to replace. It always fails, whether or not the name resolves,
#     because it has to become `#just.<name>(...)`. A bare `just` is exempt
#     (`just` with no args lists the recipes, and that sentence stays
#     literal). A just flag such as `just --fmt` is no recipe either, so it
#     gets a message of its own.
scan_sources() {
    local rows
    rows=$(doc_extract "$@") || {
        fail=1
        return
    }
    awk -F'\t' -v names_file="$names_file" '
        BEGIN {
            while ((getline n < names_file) > 0) known[n] = 1
        }
        function bad(f, ln, msg) { print "check-doc-recipes: " f ":" ln ": " msg > "/dev/stderr"; failed = 1 }
        function unknown(f, ln, name) { if (!(name in known)) bad(f, ln, "`just " name "` is not a real recipe") }
        {
            f = $1; ln = $2; kind = $3; text = $4
            if (kind != "span" && kind != "block" && kind != "prose") next
            s = text
            while (match(s, /"just [A-Za-z0-9_-]+/)) {
                name = substr(s, RSTART + 6, RLENGTH - 6)
                s = substr(s, RSTART + RLENGTH)
                if (name ~ /^-/) continue
                if (!(name in known)) bad(f, ln, "\"just " name "\" is not a real recipe")
            }
            if (kind == "block") {
                t = text; sub(/^[ \t]+/, "", t)
                if (match(t, /^just([ \t]+[A-Za-z0-9_-]+)?([ \t]|$)/)) {
                    split(t, w, /[ \t]+/)
                    if (w[2] != "" && w[2] !~ /^-/) unknown(f, ln, w[2])
                }
            } else if (kind == "span" && text ~ /^just( |$)/) {
                split(text, w, / +/)
                name = w[2]
                if (name == "") next
                if (name ~ /^-/) {
                    bad(f, ln, "`just " name "` is a just flag, not a recipe — write the flag alone")
                    next
                }
                bad(f, ln, "literal `just " name "` outside the generated module — use #just." name "(...)")
                unknown(f, ln, name)
            }
        }
        END { exit failed }
    ' <<<"$rows" || fail=1
}

# The sources come from the manifest in _doc-blocks-lib.sh. www/utils is
# scanned too: gate-notes.typ's prose renders straight into CONTRIBUTING.md,
# so a recipe name hand-typed there rots like one in a docs/src source.
doc_sources_load
sources=("${DOC_SOURCES[@]}")

# A scan that finds nothing looks the same as a scanner that reads nothing, so
# a synthetic page holding each shape runs first and its exact findings are
# asserted. Each line number names its line: the block opened mid-line (line 2)
# must hide its content, and the prose after its close (line 5) must be
# checked.
control() {
    local tmp page want got
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' RETURN
    page=$tmp/control.typ
    cat >"$page" <<'EOF'
Prose: `just check` in a span, and a bare `just` or the adverb just stay clean.
examples: ```sh
just nosuchone
`just nosuchhidden` is block content, not prose
```, then `just check` after it.
// `just nosuchcomment` in a comment line
Prose again: `just check` after the block.
A quoted "just nosuchquoted" value.
Inline ```sh just nosuchinline``` then `just check` after it.
```sh
just check
```
Flags: `just --fmt` in prose.
```sh
just --list
```
A quoted "just --fmt" value.
#let fence = "```"
After a string holding a fence: `just nosuchfence` is still prose.
EOF
    want=$(sort <<'EOF'
1: literal `just check` outside the generated module — use #just.check(...)
3: `just nosuchone` is not a real recipe
5: literal `just check` outside the generated module — use #just.check(...)
7: literal `just check` outside the generated module — use #just.check(...)
8: "just nosuchquoted" is not a real recipe
9: `just nosuchinline` is not a real recipe
9: literal `just check` outside the generated module — use #just.check(...)
13: `just --fmt` is a just flag, not a recipe — write the flag alone
19: literal `just nosuchfence` outside the generated module — use #just.nosuchfence(...)
19: `just nosuchfence` is not a real recipe
EOF
    )
    got=$(scan_sources "$page" 2>&1 | sed "s|^check-doc-recipes: $page:||" | sort)
    if [ "$got" != "$want" ]; then
        echo "check-doc-recipes: control failed; findings differ from the expected set:" >&2
        diff <(echo "$want") <(echo "$got") >&2 || true
        exit 1
    fi
}

control
scan_sources "${sources[@]}"

[ "$fail" -eq 0 ] || exit 1
