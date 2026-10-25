#!/usr/bin/env bash
# .just/scripts/check-doc-prose.sh — the body of the `docs::prose` recipe.
# The word lists and the readers that turn each source into rows are in
# _doc-prose-lib.sh. This file holds the control and the run.
# It stops doc prose, config comments and config schema descriptions from
# telling history or shouting in all-caps words, and a Typst doc source from
# typing the product's name (`name_scan` below).
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
# `EXCLUDE` names a file to skip, with its reason.
#
# It reads every `description` out of schema/config.schema.json and
# schema/at.schema.json too, because those texts reach the docs site and
# editors, and a hit there names the key path. The help strings of
# `mt _cli --json` are read the same way, labelled `cli.json:` and the command
# path. The rules it enforces, with examples, are in
# docs/src/contributing/documentation.typ under "Plain prose".
#
# One rule applies only to text a reader sees on the site, in an editor or in
# `--help`: an em dash is a hit in the rendered prose and string literals of
# www/content and www/utils, in a schema description and in a help string. A
# code comment is not that text, and neither are AGENTS.md, the skill and the
# build errors in `PAGE_ERRORS` below, which only a contributor reads.
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
# An all-caps word passes when it is an acronym in `ALLOWED`, a file name such as
# README.md, or a constant a shell file assigns, written as `$NAME`, `${NAME}`
# or `NAME=`. A lone capital letter beside an all-caps word is shouting too.
set -eo pipefail

# shellcheck source=.just/scripts/_doc-blocks-lib.sh
source .just/scripts/_doc-blocks-lib.sh
# shellcheck source=.just/scripts/_doc-prose-lib.sh
source .just/scripts/_doc-prose-lib.sh

# The files whose prose rows must hold no em dash: a regex on the file column.
# The control swaps in its own.
DASH_RE='^(www/(content|utils)/|schema/|cli[.]json:)'

# Files whose string literals may keep an em dash, each with its reason.
PAGE_ERRORS=(
    'www/utils/page.typ|build errors about a page manifest'
    'www/utils/site.typ|build errors about a route'
)

# Control: a scanner that matches nothing passes a dirty tree, so it first has
# to report exactly the hits planted in this fixture.
control() {
    local dir out want
    dir=$(mktemp -d)
    cat >"$dir/f.typ" <<'EOF'
This used
to hold. JSON passes, and so do `now()`, now() and README.md.
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
OUT=1
# It is now a SHOUTED script comment.
# OUT is shouted, but $OUT, ${OUT} and OUT= name the variable.
# A line may say NOT A GITLEAKS RULE, though A single A passes.
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
    cat >"$dir/n.typ" <<'EOF'
#let x = "Mindtape" // MindTape
Use #mindtape, not mindtape:tasks.
EOF
    cat >"$dir/m.md" <<'EOF'
Mindtape passes here, MindTape does not.
EOF
    cat >"$dir/dash.typ" <<'EOF'
// A comment — passes, and so do `a — span` and "a — quote".
One — and two — dashes count.
#let note = "A string — counts."
EOF
    cat >"$dir/dash.json" <<'EOF'
{"description": "A key — with a dash.",
 "properties": {"k": {"description": "Fine, and `a — span` passes."}}}
EOF
    cat >"$dir/cli.json" <<'EOF'
{"about": "Shout NOW — it is now fine here.",
 "commands": [{"name": "ls", "args": [{"id": "n", "about": "Two\nlines — here."}]}]}
EOF
    out=$({
        rows_for "$dir/f.typ" "$dir/g.yaml" "$dir/h.sh"
        schema_text "$dir/s.json" s.json
    } | scan "$(consts_of "$dir/h.sh")" '' '' | sed "s|^$dir/||")
    out+=$'\n'$(name_scan "$dir/n.typ" -- "$dir/m.md" | sed "s|^$dir/||")
    out+=$'\n'$({
        rows_for "$dir/dash.typ"
        string_rows "$dir/dash.typ"
        schema_text "$dir/dash.json" dash.json
        cli_text <"$dir/cli.json"
    } | scan '' "^($dir/dash|dash[.]json|cli[.]json:)" '^cli[.]json:' | sed "s|^$dir/||")
    rm -rf "$dir"
        # shellcheck disable=SC2016 # expected output, literal text
    want='f.typ:2: history word "used to"
f.typ:3: history word "now"
f.typ:3: all-caps "SHOUTED"
f.typ:8: history word "now"
f.typ:8: history word "now"
f.typ:8: history word "no longer"
f.typ:10: history word "now"
g.yaml:2: history word "now"
g.yaml:2: all-caps "SHOUTED"
h.sh:3: history word "now"
h.sh:3: all-caps "SHOUTED"
h.sh:4: all-caps "OUT"
h.sh:5: all-caps "NOT"
h.sh:5: all-caps "A"
h.sh:5: all-caps "GITLEAKS"
h.sh:5: all-caps "RULE"
h.sh:11: history word "no longer"
s.json:(root): history word "now"
s.json:$defs.A.oneOf.0: history word "used to"
s.json:$defs.A.oneOf.0: all-caps "SHOUTED"
n.typ:1: typed name "Mindtape", write #mindtape
n.typ:1: typed name "MindTape", write #mindtape
m.md:1: spelling "MindTape", write Mindtape
dash.typ:2: em dash
dash.typ:2: em dash
dash.typ:3: em dash
dash.json:(root): em dash
cli.json:about: em dash
cli.json:commands.ls.args.n.about: em dash'
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
# the kinds below, so a new script is read the day it lands. `EXCLUDE` names the
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

# The strings of the site sources, bar the files in PAGE_ERRORS.
string_sources=()
for f in "${typst_sources[@]}"; do
    skip=0
    for e in "${PAGE_ERRORS[@]}"; do
        [ "${e%%|*}" = "$f" ] && skip=1
    done
    case "$f" in www/*) [ "$skip" -eq 1 ] || string_sources+=("$f") ;; esac
done

# `mt _cli --json` comes from the same binary as check-doc-spans reads.
# `--tests`: docs/contributing/gates.md, "One mt binary, one feature set".
cargo build -p mindtape-cli --tests -q

hits=$({
    rows_for "${typst_sources[@]}" "${md_sources[@]}" "${comment_sources[@]}"
    string_rows "${string_sources[@]}"
    schema_text schema/config.schema.json schema/config.schema.json
    schema_text schema/at.schema.json schema/at.schema.json
    ./target/debug/mt _cli --json | cli_text
} | scan "$(consts_of "${typst_sources[@]}" "${comment_sources[@]}")" "$DASH_RE" '^cli[.]json:')
names=$(name_scan "${typst_sources[@]}" -- "${md_sources[@]}")
hits=$(printf '%s\n%s' "$hits" "$names" | sed '/^$/d')
if [ -n "$hits" ]; then
        # shellcheck disable=SC2001 # a prefix on every line, which ${var//} cannot anchor
    sed 's/^/check-doc-prose: /' <<<"$hits" >&2
    echo "check-doc-prose: rewrite each line plainly; the rules are in docs/contributing/documentation.md" >&2
    exit 1
fi
