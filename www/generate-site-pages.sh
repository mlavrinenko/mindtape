#!/usr/bin/env bash
# www/generate-site-pages.sh — derive www/generated/site-pages.json from
# every www/content/*.typ page's own `<page-meta>` block.
#
# Split out of www/generate-facts.sh under
# truth/tasks/derive-site-pages-json-from-the-pages-page-meta-blocks.typ: the
# manifest used to be a read-back of www/utils/site.typ's own PAGES dict
# (itself derived from NAV). Now each page carries its own
# `#metadata((route, label, title, summary, home)) <page-meta>` block, so the
# manifest is derived from the FIFTEEN PAGES instead — site.typ keeps only
# what no single page owns: the group names and the reading order (NAV).
#
# Called by www/generate.sh, AFTER its generate-facts.sh call, inside the
# lock that script already holds — this is its own file rather than more of
# generate.sh's body only because that script is already at .linecop.yaml's
# 300-line Shell cap. Ordering is not negotiable: querying a page's
# `<page-meta>` block is a FULL COMPILE (measured in the umbrella task), which
# imports layout.typ -> site.typ -> crate-map.json (PKG_VERSION) and every
# other capture www/generate.sh itself writes (hero-source.typ, cli.json, …).
# Without those a page importing utils/ui.typ dies on a top-level `read()`.
# So this script needs BOTH generate-facts.sh's artifacts and generate.sh's
# own captures in place before it queries a single page.
#
# Reentrant locking, same handshake generate-facts.sh has always used for
# generate.sh's nested call to IT: a caller that already holds
# target/www-generated.lock exports MINDTAPE_GENERATED_LOCK=1 so this script
# skips its own acquisition instead of deadlocking against a lock its own
# process tree already owns. Run standalone (as the regression
# demonstrations below do), it takes the lock itself.
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="$SCRIPT_DIR/generated"
mkdir -p "$OUT" "$REPO/target"

if [ -z "$MINDTAPE_GENERATED_LOCK" ]; then
  exec 9>"$REPO/target/www-generated.lock"
  flock 9
  export MINDTAPE_GENERATED_LOCK=1
fi

# Same temp-file-plus-atomic-mv-plus-parse-check discipline as
# generate-facts.sh's own emit() (argued in place there): a reader holding no
# lock must see the previous complete file or the next one, never a
# truncated/absent one, and a malformed producer must never install a
# plausible-looking partial artifact.
emit() {
  local name="$1" dest="$OUT/.$1.tmp.$$"
  cat >"$dest"
  if ! jq -e . "$dest" >/dev/null 2>&1; then
    rm -f "$dest"
    echo "generate-site-pages.sh: refusing to install malformed $name" >&2
    return 1
  fi
  mv -f "$dest" "$OUT/$name"
}

# Bootstrap pass. Compiling ANY page imports layout.typ -> site.typ, and
# site.typ's PAGES reads this very file, so it has to exist — even empty —
# before the query below can compile the first page. Only seeded when
# absent: a real build already has a real manifest on disk, and re-seeding it
# empty here would make every page compile against a momentarily-empty PAGES
# for no reason.
if [ ! -f "$OUT/site-pages.json" ]; then
  jq -n '{pages: {}}' | emit site-pages.json
fi

workdir="$(mktemp -d -p "$REPO/target")"
trap 'rm -rf "$workdir"' EXIT

# One `<page-meta>` query per page. `--one` makes a page with no block (or
# more than one) a hard failure; typst's own error names the file, so it is
# echoed straight through rather than swallowed. `--target html --features
# html` is required because compiling a page walks layout.typ, which calls
# `html.elem` (see docs/generate-md.sh's header for the same requirement on
# a different render path).
pages_json="{}"
routes_json="[]"
for page in "$REPO"/www/content/*.typ; do
  stem="$(basename "$page" .typ)"
  if ! meta="$(typst query --root "$REPO/www" "$page" '<page-meta>' \
      --field value --one --target html --features html 2>"$workdir/err")"; then
    echo "generate-site-pages.sh: $page has no <page-meta> block:" >&2
    cat "$workdir/err" >&2
    exit 1
  fi
  route="$(jq -r '.route' <<<"$meta")"
  pages_json="$(jq --argjson meta "$meta" --arg route "$route" \
    '. + {($route): ($meta | del(.route))}' <<<"$pages_json")"
  routes_json="$(jq --arg stem "$stem" --arg route "$route" \
    '. + [{stem: $stem, route: $route}]' <<<"$routes_json")"
done

# Two pages declaring the same route: fail naming both, before installing
# anything. The dict-insert above would otherwise let the second writer win
# silently — this is the check that makes that impossible.
dupes="$(jq -c 'group_by(.route) | map(select(length > 1))' <<<"$routes_json")"
if [ "$(jq 'length' <<<"$dupes")" -gt 0 ]; then
  echo "generate-site-pages.sh: two pages declare the same route:" >&2
  jq -r '.[] | "  " + .[0].route + ": " + (map(.stem) | join(", "))' <<<"$dupes" >&2
  exit 1
fi

# The site URL, path prefix and repo link — read back off site.typ through a
# target/-rooted driver, the same trick www/generate-facts.sh used for this
# artifact before this script existed, moved here verbatim. site.typ resolves
# its own reads relative to ITSELF rather than the root, so `--root .` here
# and `--root www` above both work against the same file. NAV's flattened
# routes ride along in the same query so the `pages` object below can be
# written out in NAV's curated reading order rather than the glob's
# alphabetical one — docs/src/readme.typ's page list reads `pages.keys()`
# directly, and that list is meant to read in the site's own order, not
# filename order.
printf '#import "/www/utils/site.typ": NAV, PREFIX, REPO, SITE_URL\n#metadata((url: SITE_URL, prefix: PREFIX, repo: REPO, nav: NAV.map(g => g.items).flatten())) <site-meta>\n' \
  >"$workdir/site-meta-driver.typ"
site_meta="$(typst query --root "$REPO" "$workdir/site-meta-driver.typ" '<site-meta>' \
  --field value --one)"

jq -n --argjson site "$site_meta" --argjson pages "$pages_json" '
  ($site.nav | map(select(. as $r | $pages | has($r)))) as $ordered
  | (($pages | keys) - $ordered) as $rest
  | {
      url: $site.url,
      prefix: $site.prefix,
      repo: $site.repo,
      pages: (reduce ($ordered + $rest)[] as $r ({}; . + {($r): $pages[$r]})),
    }
' | emit site-pages.json

echo "generate-site-pages.sh: $(jq 'length' <<<"$pages_json") pages"
