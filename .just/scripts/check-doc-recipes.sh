#!/usr/bin/env bash
# .just/scripts/check-doc-recipes.sh — the body of the `check-doc-recipes`
# recipe, extracted under truth/tasks/extract-the-long-gate-recipe-bodies-into-
# scripts.typ. Every path below is repo-root-relative, because the recipe that
# invokes it runs at the repo root; the prose arguing what this gate is for —
# the three shapes of literal mention it matches, and why each is or is not
# legal — stays above that recipe in the Justfile.
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
# www/utils/ is scanned too: gate-notes.typ's prose is rendered straight
# into CONTRIBUTING.md, so a recipe name hand-typed there rots exactly
# like one in a docs/src source. The other notes dicts under www/utils
# mention no recipe today and cost nothing to scan.
sources=(docs/src/**/*.typ www/content/*.typ www/utils/*.typ)
fail=0

for f in "${sources[@]}"; do
    [ "$(basename "$f")" = "lib.typ" ] && continue

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

        while IFS= read -r m; do
            [ -z "$m" ] && continue
            name="${m#\"just }"
            if ! is_recipe "$name"; then
                echo "check-doc-recipes: $f:$lineno: \"just $name\" is not a real recipe" >&2
                fail=1
            fi
        done < <(grep -oE '"just [A-Za-z0-9_-]+' <<<"$content" || true)

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
echo "check-doc-recipes: no hand-typed just mention skips the generated module"
