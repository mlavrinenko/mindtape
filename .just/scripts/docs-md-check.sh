#!/usr/bin/env bash
# .just/scripts/docs-md-check.sh — the body of the `md-check` recipe in
# .just/docs.just (invoked as `just docs::md-check`, mangled to `docs-md-check`
# everywhere the docs name it), extracted under truth/tasks/extract-the-long-
# gate-recipe-bodies-into-scripts.typ. Every path below is repo-root-relative:
# that module file carries `set working-directory := '..'` precisely so its
# recipes run at the repo root rather than in .just/. The prose arguing why the
# regenerate goes to a TEMP directory stays above that recipe.
set -eo pipefail
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
bash docs/generate-md.sh "$tmp"
drift=()
while IFS= read -r -d '' f; do
    rel="${f#"$tmp"/}"
    if [ ! -f "$rel" ] || ! diff -q "$rel" "$f" >/dev/null 2>&1; then
        drift+=("$rel")
    fi
done < <(find "$tmp" -type f -print0)
if [ "${#drift[@]}" -gt 0 ]; then
    echo "docs::md-check: drifted from docs/src/ (run \`just docs::md\` to refresh):" >&2
    for f in "${drift[@]}"; do echo "  - $f" >&2; done
    exit 1
fi
