#!/usr/bin/env bash
# .just/scripts/check-doc-coverage.sh — the body of the `docs::coverage`
# recipe. Every path below is repo-root-relative, because the recipe runs at
# the repo root.
#
# Every visible top-level `mt` command needs a www/utils/cli-notes.typ entry
# and a page at www/content/cli/<command>.typ, and every page there needs a
# command. Without the gate a new command ships with no prose and no page, and
# only a reviewer would notice. Each arm is a set difference against the names
# `mt _cli --json` reflects, the same dump the CLI pages render.
set -eo pipefail
# `--tests`, not a plain build: the test arm links `target/debug/mt` with the
# dev-dependency features, and a build without them would relink it away
# mid-run (docs/contributing/gates.md, "One mt binary, one feature set").
cargo build -p mindtape-cli --tests -q
# cli/tree_json.rs drops every `hide = true` command and clap's `help` before they
# reach this JSON, so the gate needs no skip list of its own.
visible=$(./target/debug/mt _cli --json | jq -r '.commands[].name' | sort)

# cli-notes.typ imports www/utils/site.typ, which reads www/generated/ at
# module top level. The docs arms of `just check` rewrite that directory in
# parallel with this one, so the query waits on the lock they hold while they
# write. This script writes nothing there.
mkdir -p target
exec 9>target/www-generated.lock
flock 9
export MINDTAPE_GENERATED_LOCK=1

# The note keys come from evaluating the Typst dict, because a regex for
# quoted strings would also match a dotted argument key like "ls.sort".
# `typst query` reads document content only, so a driver under target/ (inside
# `--root .`, as typst requires) re-exports the keys as metadata.
tmp=$(mktemp -d -p target)
trap 'rm -rf "$tmp"' EXIT
printf '#import "/www/utils/cli-notes.typ": cli-notes\n#metadata(cli-notes.keys()) <cli-notes-keys>\n' \
    > "$tmp/keys.typ"
documented=$(typst query --root . "$tmp/keys.typ" '<cli-notes-keys>' \
    --field value --one | jq -r '.[]' | sort)
shopt -s nullglob
paged=$(for f in www/content/cli/*.typ; do basename "$f" .typ; done | sort)

fail=0
report() {
    local title="$1" names="$2"
    [ -n "$names" ] || return 0
    echo "check-doc-coverage: $title" >&2
    while IFS= read -r name; do echo "  - $name" >&2; done <<<"$names"
    fail=1
}
report "commands with no www/utils/cli-notes.typ entry:" \
    "$(comm -23 <(echo "$visible") <(echo "$documented") | sed 's/^/mt /')"
report "commands with no page (add www/content/cli/<command>.typ):" \
    "$(comm -23 <(echo "$visible") <(echo "$paged") | sed 's/^/mt /')"
report "pages under www/content/cli/ that name no command:" \
    "$(comm -13 <(echo "$visible") <(echo "$paged") | sed 's|^\(.*\)$|www/content/cli/\1.typ|')"
exit "$fail"
