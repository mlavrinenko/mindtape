#!/usr/bin/env bash
# .just/scripts/check-doc-facts.sh — the body of the `check-doc-facts` recipe,
# extracted under truth/tasks/extract-the-long-gate-recipe-bodies-into-
# scripts.typ. Every path below is repo-root-relative, because the recipe that
# invokes it runs at the repo root.
#
# Doc-facts gate: the same set-difference idiom as check-doc-coverage.sh, over
# the facts www/generate-facts.sh derives instead of a hand-maintained list
# (see truth/tasks/derive-the-drift-prone-doc-facts-from-their-sources.typ).
# Four arms: every crate in crate-map.json needs a www/utils/crate-notes.typ
# entry, every DSL field in dsl-fields.json needs a
# www/utils/dsl-field-notes.typ entry, every gate in gates.json needs a
# www/utils/gate-notes.typ entry (truth/tasks/derive-the-gate-table-command-
# column-from-the-justfile.typ), and every sweep flag in sweep-flags.json needs
# a www/utils/sweep-flag-notes.typ entry (truth/tasks/derive-the-ask-
# refactoring-flag-table-from-arg-attributes.typ). Each arm reads its Typst
# dict's keys through the same target/-rooted driver trick as
# check-doc-coverage.sh, never by grepping quoted strings.
#
# It regenerates its own inputs (bash www/generate-facts.sh below) rather than
# trusting whatever www/generated/ already holds, so this gate never depends
# on `just docs::gen` — or a release build — having run first.
set -eo pipefail
# Hold the www/generated/ lock across the regenerate AND the typst queries
# below, not just the regenerate: www/utils/gate-notes.typ reads the
# generated `just` module, so a driver querying it is a reader of that
# directory, and the concurrent docs::check arm `rm -rf`s it. Same fd-held
# lock and same MINDTAPE_GENERATED_LOCK handshake as www/generate.sh and
# docs/generate-md.sh.
mkdir -p target
exec 9>target/www-generated.lock
flock 9
export MINDTAPE_GENERATED_LOCK=1
bash www/generate-facts.sh
tmp=$(mktemp -d -p target)
trap 'rm -rf "$tmp"' EXIT

crates=$(jq -r '.workspace[].name, .substrate[].name' www/generated/crate-map.json)
printf '#import "/www/utils/crate-notes.typ": crate-notes\n#metadata(crate-notes.keys()) <crate-notes-keys>\n' \
    > "$tmp/crate-keys.typ"
documented_crates=$(typst query --root . "$tmp/crate-keys.typ" '<crate-notes-keys>' \
    --field value --one | jq -r '.[]')
missing_crates=$(comm -23 <(sort <<<"$crates") <(sort <<<"$documented_crates"))
if [ -n "$missing_crates" ]; then
    echo "check-doc-facts: undocumented crates (no www/utils/crate-notes.typ entry):" >&2
    while IFS= read -r name; do echo "  - $name" >&2; done <<<"$missing_crates"
    exit 1
fi

fields=$(jq -r '.fields[].name' www/generated/dsl-fields.json)
printf '#import "/www/utils/dsl-field-notes.typ": dsl-field-notes\n#metadata(dsl-field-notes.keys()) <dsl-field-notes-keys>\n' \
    > "$tmp/field-keys.typ"
documented_fields=$(typst query --root . "$tmp/field-keys.typ" '<dsl-field-notes-keys>' \
    --field value --one | jq -r '.[]')
missing_fields=$(comm -23 <(sort <<<"$fields") <(sort <<<"$documented_fields"))
if [ -n "$missing_fields" ]; then
    echo "check-doc-facts: undocumented DSL fields (no www/utils/dsl-field-notes.typ entry):" >&2
    while IFS= read -r name; do echo "  - $name" >&2; done <<<"$missing_fields"
    exit 1
fi

gates=$(jq -r '.gates[].name' www/generated/gates.json)
printf '#import "/www/utils/gate-notes.typ": gate-notes\n#metadata(gate-notes.keys()) <gate-notes-keys>\n' \
    > "$tmp/gate-keys.typ"
documented_gates=$(typst query --root . "$tmp/gate-keys.typ" '<gate-notes-keys>' \
    --field value --one | jq -r '.[]')
missing_gates=$(comm -23 <(sort <<<"$gates") <(sort <<<"$documented_gates"))
if [ -n "$missing_gates" ]; then
    echo "check-doc-facts: undocumented gates (no www/utils/gate-notes.typ entry):" >&2
    while IFS= read -r name; do echo "  - $name" >&2; done <<<"$missing_gates"
    exit 1
fi
flags=$(jq -r '.flags[].long' www/generated/sweep-flags.json)
printf '#import "/www/utils/sweep-flag-notes.typ": sweep-flag-notes\n#metadata(sweep-flag-notes.keys()) <sweep-flag-notes-keys>\n' \
    > "$tmp/sweep-flag-keys.typ"
documented_flags=$(typst query --root . "$tmp/sweep-flag-keys.typ" '<sweep-flag-notes-keys>' \
    --field value --one | jq -r '.[]')
missing_flags=$(comm -23 <(sort <<<"$flags") <(sort <<<"$documented_flags"))
if [ -n "$missing_flags" ]; then
    echo "check-doc-facts: undocumented sweep flags (no www/utils/sweep-flag-notes.typ entry):" >&2
    while IFS= read -r name; do echo "  - $name" >&2; done <<<"$missing_flags"
    exit 1
fi
echo "check-doc-facts: $(wc -l <<<"$crates") crates, $(wc -l <<<"$fields") DSL fields, $(wc -l <<<"$gates") gates, $(wc -l <<<"$flags") sweep flags, all documented"
