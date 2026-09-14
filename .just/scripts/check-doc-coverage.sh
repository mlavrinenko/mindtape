#!/usr/bin/env bash
# .just/scripts/check-doc-coverage.sh — the body of the `check-doc-coverage`
# recipe, extracted under truth/tasks/extract-the-long-gate-recipe-bodies-into-
# scripts.typ. Every path below is repo-root-relative, because the recipe that
# invokes it runs at the repo root.
#
# Doc-coverage gate: every visible top-level `mt` subcommand must have a
# www/utils/cli-notes.typ entry. `mt find` once shipped with none — the docs
# site would have carried a new verb with no prose or examples, and only a
# human reviewing dependents caught it, which is not a mechanism. The check is
# a set-difference: `mt _cli --json`'s reflected command names (the same dump
# cli.typ itself renders) minus cli-notes.typ's dict keys.
set -eo pipefail
cargo build -p mindtape-cli -q
# Hidden/internal commands (`_cli` itself, the deprecated `install-lib` alias)
# are excluded because cli_dump.rs (`crates/mindtape-cli/src/cli_dump.rs`)
# already drops any `hide = true` command and clap's synthetic `help` pseudo-
# command before they ever reach this JSON — never by a hand-maintained skip
# list here, which would rot exactly the way the thing being gated rots.
visible=$(./target/debug/mt _cli --json | jq -r '.commands[].name')

# cli-notes.typ imports www/utils/site.typ for `u`, and site.typ reads BOTH
# www/generated/crate-map.json (PKG_VERSION) and www/generated/site-pages.json
# (PAGES) at module top level — Typst evaluates a whole module on import, not
# just the names pulled out of it. This gate never produces either file
# itself, but three OTHER `just check` arms do (docs::check, check-doc-facts,
# docs/generate-md.sh — all via `www/generate.sh`, which `rm -rf`s the whole
# directory before repopulating it), and `just check` runs them in parallel
# with this one. Reading with no lock of its own, this query could land mid-
# wipe and fail "file not found" on a tree that a moment later is perfectly
# correct — the same race `www/generate.sh`'s own header argues against for
# ITS readers. Same fd-held lock and MINDTAPE_GENERATED_LOCK handshake as
# those three; unlike them, this script produces nothing, so it only waits
# for whichever of them is mid-write, never regenerates anything itself.
mkdir -p target
exec 9>target/www-generated.lock
flock 9
export MINDTAPE_GENERATED_LOCK=1

# Reading cli-notes.typ's keys means actually evaluating the Typst dict, not
# grepping for quoted strings (a dotted arg-level key like "ls.sort" would
# false-positive a regex meant to only see top-level command names, and a
# regex silently drifts the moment the file's shape changes). `typst query`
# only inspects document content, and cli-notes.typ is a bare `#let` with
# none, so this throwaway driver imports it and re-exports `cli-notes.keys()`
# as `metadata` a label query can select. The driver must live inside
# `--root .` (typst refuses a source outside the project root), so it goes
# under target/ — gitignored, and already there after the build above.
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
