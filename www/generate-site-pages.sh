#!/usr/bin/env bash
# www/generate-site-pages.sh — derive www/generated/site-pages.json from
# every www/content/**/*.typ page's own
# `#metadata((route, label, title, summary, home)) <page-meta>` block.
# www/utils/site.typ keeps only what no single page holds: the nav groups and
# their reading order (`NAV`).
#
# www/generate.sh calls this after generate-facts.sh, inside the lock it
# already holds. Querying a page's `<page-meta>` block is a full compile,
# which reads crate-map.json and the captures www/generate.sh writes, so both
# must exist before the first query.
#
# A caller that already holds target/www-generated.lock exports
# MINDTAPE_GENERATED_LOCK=1, and this script skips taking the lock again so it
# cannot deadlock against its own process tree. Run standalone, it takes the
# lock itself.
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

# Bootstrap pass. Compiling any page imports layout.typ -> site.typ, and
# site.typ's `PAGES` reads this very file, so it has to exist — even empty —
# before the query below can compile the first page. Only seeded when
# absent: a real build already has a real manifest on disk, and re-seeding it
# empty here would make every page compile against a momentarily-empty `PAGES`
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
summary_problems=""
shopt -s globstar
for page in "$REPO"/www/content/**/*.typ; do
  stem="${page#"$REPO"/www/content/}"
  stem="${stem%.typ}"
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
  # The summary is the page's lede and its <meta name="description">, so it
  # stays one short sentence. Backticks count for nothing, and a `.` inside a
  # code span ends no sentence.
  problem="$(jq -r '(.summary // "") as $s | [
      ($s | gsub("`"; "") | length | select(. > 120) | "\(.) characters"),
      ($s | gsub("`[^`]*`"; "code") | select(test("[.!?]\\s+\\S"))
        | "more than one sentence")
    ] | join(", ")' <<<"$meta")"
  if [ -n "$problem" ]; then
    summary_problems+="  $stem: $problem"$'\n'
  fi
done

if [ -n "$summary_problems" ]; then
  echo "generate-site-pages.sh: a page summary must be one sentence of at most 120 characters:" >&2
  printf '%s' "$summary_problems" >&2
  exit 1
fi

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
# its own reads relative to itself rather than the root, so `--root .` here
# and `--root www` above both work against the same file. `NAV` rides along in
# the same query in two shapes. The flattened routes order the `pages` object
# below by `NAV`'s reading order rather than the glob's alphabetical one, and
# `groups` carries each `NAV` group's title and routes in order, which
# docs/src/readme.typ's index reads.
printf '#import "/www/utils/site.typ": NAV, PREFIX, REPO, SITE_URL\n#metadata((url: SITE_URL, prefix: PREFIX, repo: REPO, nav: NAV.map(g => g.items).flatten(), groups: NAV.map(g => (title: g.title, routes: g.items)))) <site-meta>\n' \
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
      groups: $site.groups,
      pages: (reduce ($ordered + $rest)[] as $r ({}; . + {($r): $pages[$r]})),
    }
' | emit site-pages.json
