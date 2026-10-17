#!/usr/bin/env bash
# .just/scripts/check-doc-facts.sh — the body of the `check-doc-facts` recipe,
# extracted under truth/tasks/extract-the-long-gate-recipe-bodies-into-
# scripts.typ. Every path below is repo-root-relative, because the recipe that
# invokes it runs at the repo root.
#
# Doc-facts gate: the same set-difference idiom as check-doc-coverage.sh, over
# the facts www/generate-facts.sh and www/generate-site-pages.sh derive
# instead of a hand-maintained list (see truth/tasks/derive-the-drift-prone-
# doc-facts-from-their-sources.typ and truth/tasks/derive-site-pages-json-
# from-the-pages-page-meta-blocks.typ). Eight arms: every crate in
# crate-map.json needs a www/utils/crate-notes.typ entry, every DSL field in
# dsl-fields.json needs a www/utils/dsl-field-notes.typ entry, every priority framework
# in frameworks.json needs a www/utils/framework-notes.typ entry
# (truth/tasks/derive-the-built-in-priority-tables.typ), every sweep flag in
# sweep-flags.json needs a www/utils/sweep-flag-notes.typ entry
# (truth/tasks/derive-the-ask-refactoring-flag-table-from-arg-attributes.typ),
# every slot kind in slot-kinds.json needs a www/utils/slot-kind-notes.typ entry,
# every filter field in filter-fields.json opens a code span in the skill's
# filter reference,
# and every www/content/**/*.typ page needs a route in site-pages.json and,
# when top-level, a `NAV` entry in www/utils/site.typ. The first four arms read their Typst dict's
# keys through the same target/-rooted driver trick as check-doc-coverage.sh,
# never by grepping quoted strings. The pages arm reads a driver too (for
# `NAV`) and also compares against the filesystem, so it runs in three
# directions — a page whose own declared route disagrees with its filename, a
# page with no `NAV` entry (invisible in the sidebar), and a `NAV` entry with no
# page (a 404 in it) — each failing by name.
#
# It regenerates its own inputs through `mmz just docs::gen` (www/generate.sh,
# which ends by calling generate-facts.sh and then generate-site-pages.sh
# itself) rather than trusting whatever www/generated/ already holds, so this
# gate never depends on another recipe having run first. Deriving
# site-pages.json is a full page compile that needs generate.sh's own captures
# (hero-source.typ, cli.json, …), so calling generate-facts.sh directly cannot
# work; see that script's header. Inside `just check` the regenerate is
# usually a cache hit: whichever docs gate takes target/www-generated.lock
# first runs the generator, and mmz finds the record fresh for the rest.
set -eo pipefail
# Hold the www/generated/ lock across the regenerate and the typst queries
# below, not just the regenerate: www/utils/gate-notes.typ reads the
# generated `just` module, so a driver querying it is a reader of that
# directory, and the concurrent docs::check arm `rm -rf`s it. Same fd-held
# lock and same MINDTAPE_GENERATED_LOCK handshake as www/generate.sh and
# docs/generate-md.sh — and the reason www/generate.sh's own lock acquisition
# below is conditional on that handshake rather than unconditional: without
# it, generate.sh trying to take a second lock on the file this fd already
# holds would deadlock against this process.
mkdir -p target
exec 9>target/www-generated.lock
flock 9
export MINDTAPE_GENERATED_LOCK=1
mmz just docs::gen
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

frameworks=$(jq -r '.frameworks[].name' www/generated/frameworks.json)
printf '#import "/www/utils/framework-notes.typ": framework-notes\n#metadata(framework-notes.keys()) <framework-notes-keys>\n' \
    > "$tmp/framework-keys.typ"
documented_frameworks=$(typst query --root . "$tmp/framework-keys.typ" '<framework-notes-keys>' \
    --field value --one | jq -r '.[]')
missing_frameworks=$(comm -23 <(sort <<<"$frameworks") <(sort <<<"$documented_frameworks"))
if [ -n "$missing_frameworks" ]; then
    echo "check-doc-facts: undocumented priority frameworks (no www/utils/framework-notes.typ entry):" >&2
    while IFS= read -r name; do echo "  - $name" >&2; done <<<"$missing_frameworks"
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

# The filter-functions arm: every predicate and function in the filter
# registry (crates/mindtape/src/filter/eval.rs, read back through
# filter-functions.json) must open a code span on both filter pages. A span
# opens with the name followed by a non-name character, so `has_tag("x")` does
# not document `has`.
fns=$(jq -r '.functions[].name' www/generated/filter-functions.json)
for page in www/content/filters.typ skills/mindtape/references/filters.md; do
    missing_fns=$(while IFS= read -r name; do
        grep -qE "\`$name([^A-Za-z0-9_-]|\$)" "$page" || echo "$name"
    done <<<"$fns")
    if [ -n "$missing_fns" ]; then
        echo "check-doc-facts: filter functions missing from $page:" >&2
        while IFS= read -r name; do echo "  - $name" >&2; done <<<"$missing_fns"
        exit 1
    fi
done

# The filter-fields arm: every field in the filter field registry (`FIELDS` in
# crates/mindtape/src/model/fields.rs, read back through filter-fields.json)
# must open a code span in the skill's filter reference, which an agent reads
# with no site access. The site's table renders from the same fact, and its
# build fails on a field with no www/utils/filter-field-notes.typ entry. A
# field name holds dots, so they're escaped before the name becomes a pattern.
filter_fields=$(jq -r '.fields[].name' www/generated/filter-fields.json)
page=skills/mindtape/references/filters.md
missing_filter_fields=$(while IFS= read -r name; do
    grep -qE "\`${name//./\\.}([^A-Za-z0-9_.-]|\$)" "$page" || echo "$name"
done <<<"$filter_fields")
if [ -n "$missing_filter_fields" ]; then
    echo "check-doc-facts: filter fields missing from $page:" >&2
    while IFS= read -r name; do echo "  - $name" >&2; done <<<"$missing_filter_fields"
    exit 1
fi

# The slot-kinds arm: every kind in the SlotKind registry (read back through
# slot-kinds.json) needs a www/utils/slot-kind-notes.typ entry, so the
# templates page never renders a kind without its note.
kinds=$(jq -r '.kinds[].name' www/generated/slot-kinds.json)
printf '#import "/www/utils/slot-kind-notes.typ": slot-kind-notes\n#metadata(slot-kind-notes.keys()) <slot-kind-notes-keys>\n' \
    > "$tmp/slot-kind-keys.typ"
documented_kinds=$(typst query --root . "$tmp/slot-kind-keys.typ" '<slot-kind-notes-keys>' \
    --field value --one | jq -r '.[]')
missing_kinds=$(comm -23 <(sort <<<"$kinds") <(sort <<<"$documented_kinds"))
if [ -n "$missing_kinds" ]; then
    echo "check-doc-facts: undocumented slot kinds (no www/utils/slot-kind-notes.typ entry):" >&2
    while IFS= read -r name; do echo "  - $name" >&2; done <<<"$missing_kinds"
    exit 1
fi

# The completions arm: every value-taking argument clap declares must name
# where its values come from (truth/tasks/emit-shell-completions-from-the-
# binary-that-answers-them.typ). Both sides come out of cli.json, which is
# `mt _cli --json` — so the "declared" side is the live parser and the
# "sourced" side is crates/mindtape-cli/src/cli/completion/mod.rs's `SOURCES` table
# reflected back through it. Reading the table through the binary rather than
# grepping the file is what makes the two sides comparable: a mention the
# parser never sees cannot satisfy this, and an argument renamed in one place
# fails rather than quietly matching nothing.
#
# Two exclusions, neither a skip list. A `flag` takes no value, so there is
# nothing to complete; and an argument with `possible_values` is a ValueEnum,
# which clap completes from the parser itself. Naming those in `SOURCES` would
# restate a list the parser already owns, the committed-script mistake this
# whole feature exists to avoid.
#
# `Source::None` counts as sourced: it is a recorded decision carrying its
# reason (cli.json shows it as a null source with a non-null reason), and the
# unit test in cli/completion/tests.rs is what holds those reasons non-empty. What
# fails here is the third state — an argument the table does not name at all,
# which is exactly what a new completable argument looks like on the commit
# that adds it.
# shellcheck disable=SC2016 # a jq program, whose $names are jq's
completable='[ (.globals[] | .path = "") ]
  + [ .commands[] | .name as $c | (.args[] | .path = $c) ]
  + [ .commands[] | .name as $c | .subcommands[] | .name as $s
      | (.args[] | .path = "\($c) \($s)") ]
  | .[] | select(.kind != "flag") | select(.possible_values == null)'
declared_args=$(jq -r "$completable"' | "\(.path) \(.id)"' www/generated/cli.json)
sourced_args=$(jq -r "$completable"' | select(.completion != null) | "\(.path) \(.id)"' \
    www/generated/cli.json)
missing_args=$(comm -23 <(sort <<<"$declared_args") <(sort <<<"$sourced_args"))
if [ -n "$missing_args" ]; then
    echo "check-doc-facts: arguments with no completion value source" \
        "(no crates/mindtape-cli/src/cli/completion/mod.rs SOURCES entry):" >&2
    while IFS= read -r name; do echo "  - mt $name" >&2; done <<<"$missing_args"
    exit 1
fi

# The pages arm, part one: filename vs. the route each page declares.
# site-pages.json is built by www/generate-site-pages.sh from every page's own
# `<page-meta>` block, keyed by the route that block declares, not by the
# page's filename. So this diff is "file vs. the page's own claim about its
# route". Comparing that against the filename catches a `<page-meta>` route
# pointed at another page's route (www/generate-site-pages.sh already refuses
# two pages declaring the same route; this catches one declaring the wrong one,
# which is a valid route
# that simply is not this file's). The route a file maps to is tola's own
# layout rule — index.typ serves "/" and every other <path>.typ under
# www/content/ serves "/<path>/" — applied here rather than read from
# anywhere, because the whole point is to catch the file and its own declared
# route disagreeing.
routes=$(jq -r '.pages | keys_unsorted[]' www/generated/site-pages.json)
page_routes=$(shopt -s globstar; for f in www/content/**/*.typ; do
    stem=${f#www/content/}
    stem=${stem%.typ}
    if [ "$stem" = "index" ]; then echo "/"; else echo "/$stem/"; fi
done)
unlisted=$(comm -23 <(sort <<<"$page_routes") <(sort <<<"$routes"))
if [ -n "$unlisted" ]; then
    echo "check-doc-facts: content pages whose declared route matches no filename-derived route:" >&2
    while IFS= read -r route; do echo "  - $route" >&2; done <<<"$unlisted"
    exit 1
fi
orphaned=$(comm -13 <(sort <<<"$page_routes") <(sort <<<"$routes"))
if [ -n "$orphaned" ]; then
    echo "check-doc-facts: filename-derived routes with no page declaring them:" >&2
    while IFS= read -r route; do echo "  - $route" >&2; done <<<"$orphaned"
    exit 1
fi

# The pages arm, part two: the manifest vs. www/utils/site.typ's `NAV`, the
# other place a route is written by hand. Nothing derives `NAV` from the
# manifest any more (`NAV` only carries group titles and reading order), so the
# two can disagree — a page missing from `NAV` is invisible in the sidebar, and
# a `NAV` entry naming no real page's route 404s the moment it's clicked. Read
# through the same target/-rooted driver trick as the four arms above, not by
# grepping quoted strings out of site.typ. The set-difference itself is
# .just/scripts/_nav-diff.sh, extracted under truth/tasks/pin-the-docs-
# manifest-gate-failures-with-a-selftest.typ so site-pages-fixture-check.sh
# can pin it against a synthetic pair of route lists without paying for this
# regenerate.
printf '#import "/www/utils/site.typ": NAV\n#metadata(NAV.map(g => g.items).flatten()) <nav-routes>\n' \
    > "$tmp/nav-routes.typ"
nav_routes=$(typst query --root . "$tmp/nav-routes.typ" '<nav-routes>' \
    --field value --one | jq -r '.[]')
# A nested page, such as /cli/ls/, is reached from its parent page and shares
# its parent's sidebar entry, so `NAV` lists top-level routes only. Its parent
# has to be a `NAV` route, or nothing in the sidebar leads to it.
top_routes=$(grep -E '^/([^/]+/)?$' <<<"$routes")
strays=$(grep -vE '^/([^/]+/)?$' <<<"$routes" | while IFS= read -r route; do
    parent=$(sed -E 's|[^/]+/$||' <<<"$route")
    grep -qxF "$parent" <<<"$nav_routes" || echo "$route"
done)
if [ -n "$strays" ]; then
    echo "check-doc-facts: nested pages whose parent route is not in www/utils/site.typ's NAV:" >&2
    while IFS= read -r route; do echo "  - $route" >&2; done <<<"$strays"
    exit 1
fi
bash .just/scripts/_nav-diff.sh "$top_routes" "$nav_routes"
