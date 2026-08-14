#!/usr/bin/env bash
# .just/scripts/check-doc-coverage.sh — the body of the `check-doc-coverage`
# recipe, extracted under truth/tasks/extract-the-long-gate-recipe-bodies-into-
# scripts.typ. Every path below is repo-root-relative, because the recipe that
# invokes it runs at the repo root; the prose arguing what this gate is for
# stays above that recipe in the Justfile.
set -eo pipefail
cargo build -p mindtape-cli -q
visible=$(./target/debug/mt _cli --json | jq -r '.commands[].name')
tmp=$(mktemp -d -p target)
trap 'rm -rf "$tmp"' EXIT
printf '#import "/www/utils/cli-notes.typ": cli-notes\n#metadata(cli-notes.keys()) <cli-notes-keys>\n' \
    > "$tmp/keys.typ"
documented=$(typst query --root . "$tmp/keys.typ" '<cli-notes-keys>' \
    --field value --one | jq -r '.[]')
missing=$(comm -23 <(sort <<<"$visible") <(sort <<<"$documented"))
if [ -n "$missing" ]; then
    echo "check-doc-coverage: undocumented commands (no www/utils/cli-notes.typ entry):" >&2
    while IFS= read -r name; do echo "  - mt $name" >&2; done <<<"$missing"
    exit 1
fi
echo "check-doc-coverage: $(wc -l <<<"$visible") visible commands, all documented"
