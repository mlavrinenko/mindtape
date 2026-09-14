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
is_recipe() { grep -qxF "$1" <<<"$names"; }

shopt -s globstar nullglob
# Scans docs/src/**/*.typ — skipping lib.typ, a helper module rather than a
# source; its own mentions are in `//` comments, invisible to every render
# path — and www/content/*.typ. www/utils/ is scanned too: gate-notes.typ's
# prose is rendered straight into CONTRIBUTING.md, so a recipe name hand-typed
# there rots exactly like one in a docs/src source. The other notes dicts
# under www/utils mention no recipe today and cost nothing to scan.
sources=(docs/src/**/*.typ www/content/*.typ www/utils/*.typ)
fail=0

for f in "${sources[@]}"; do
    [ "$(basename "$f")" = "lib.typ" ] && continue

    # Tag every line by syntactic role first: a fence delimiter, a full `//`
    # comment line (never rendered, so never checked — same reason lib.typ's
    # own mentions are exempt), inside a fenced block, or plain prose. The
    # three shapes of literal mention below follow from that tagging, and are
    # matched independently.
    tagged=$(awk '
        BEGIN { fence = 0 }
        {
            t = $0
            sub(/^[ \t]+/, "", t)
            if (t ~ /^```/) { fence = !fence; print "F\t" NR "\t" $0; next }
            if (t ~ /^\/\//) { print "C\t" NR "\t" $0; next }
            print (fence ? "B" : "P") "\t" NR "\t" $0
        }
    ' "$f")

    while IFS=$'\t' read -r tag lineno content; do
        [ "$tag" = "F" ] && continue
        [ "$tag" = "C" ] && continue

        # Shape three: a quoted `"just <name>"` substring anywhere — a literal
        # value, not a live reference (an example `.mindtape/config.toml` flip
        # hook's `run = "just check"`, a demonstrated status-note string).
        # Never has to convert, but still has to resolve.
        while IFS= read -r m; do
            [ -z "$m" ] && continue
            name="${m#\"just }"
            if ! is_recipe "$name"; then
                echo "check-doc-recipes: $f:$lineno: \"just $name\" is not a real recipe" >&2
                fail=1
            fi
        done < <(grep -oE '"just [A-Za-z0-9_-]+' <<<"$content" || true)

        # Shape two: `just <name>` opening a line INSIDE a fenced block — a
        # shell-command example (Quick Start's `just check`, the signal
        # sweep's dry-run transcript). A fenced block cannot call the helper,
        # so this is legal as prose, but the name still has to resolve.
        if [ "$tag" = "B" ]; then
            trimmed="${content#"${content%%[![:space:]]*}"}"
            if [[ "$trimmed" =~ ^just([[:space:]]+([A-Za-z0-9_-]+))?([[:space:]]|$) ]]; then
                name="${BASH_REMATCH[2]}"
                if [ -n "$name" ] && ! is_recipe "$name"; then
                    echo "check-doc-recipes: $f:$lineno: \`just $name\` is not a real recipe" >&2
                    fail=1
                fi
            fi
            continue
        fi

        # Shape one: a backtick span opening with `just` in PROSE — the
        # hand-typed mention the generated module exists to replace. Always
        # fails, whether or not the name resolves: it has to become
        # `#just.<name>(...)` either way. A bare `just` with nothing after it
        # is exempt (`just` with no args lists the recipes, and that sentence
        # has to stay literal).
        while IFS= read -r m; do
            [ -z "$m" ] && continue
            inner="${m#\`}"; inner="${inner%\`}"
            name=""
            read -r _ name _ <<<"$inner"
            [ -z "$name" ] && continue
            echo "check-doc-recipes: $f:$lineno: literal \`just $name\` outside the generated module — use #just.$name(...)" >&2
            fail=1
            if ! is_recipe "$name"; then
                echo "check-doc-recipes: $f:$lineno: \`just $name\` is not a real recipe" >&2
            fi
        done < <(grep -oE '`just(`| [^`]*`)' <<<"$content" || true)
    done <<<"$tagged"
done

[ "$fail" -eq 0 ] || exit 1
