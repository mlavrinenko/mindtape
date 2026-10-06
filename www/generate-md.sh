#!/usr/bin/env bash
# www/generate-md.sh — write the Markdown copy of every docs page, and
# /llms.txt, into the built site.
#
# Each www/content page renders a second time with typlite and
# `--input markdown=1` (www/utils/mode.typ), into `<route>index.md` beside the
# page's HTML. In that mode the page renders its title, summary and body with
# no tola wrapper, and its links point at the other pages' copies. llms.txt
# follows llmstxt.org: the site title and summary, then each `NAV` group's pages
# as `[title](md url): summary`, with a nested page such as /cli/ls/ listed
# under its parent's entry.
#
# The build fails on a page typlite cannot render, on an HTML tag typlite
# wrote because a construct has no Markdown form, and on a link to a Markdown
# copy that does not exist.
#
# Usage: www/generate-md.sh [SITE_DIR]. SITE_DIR defaults to the directory
# `tola build` writes, www/public plus the site's path prefix. Run it after
# `tola build`, because `tola build --clean` empties that directory.
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
GEN="$SCRIPT_DIR/generated"

# Every page reads www/generated/, which www/generate.sh empties on entry, so
# this script holds the lock while it renders. A caller that holds it already
# exports MINDTAPE_GENERATED_LOCK=1.
mkdir -p "$REPO/target"
if [ -z "$MINDTAPE_GENERATED_LOCK" ]; then
  exec 9>"$REPO/target/www-generated.lock"
  flock 9
  export MINDTAPE_GENERATED_LOCK=1
fi

SITE_URL="$(jq -r '.url' "$GEN/site-pages.json")"
OUT="${1:-$SCRIPT_DIR/public$(jq -r '.prefix' "$GEN/site-pages.json")}"
if [ ! -d "$OUT" ]; then
  echo "www/generate-md.sh: $OUT does not exist; run tola build first" >&2
  exit 1
fi

work="$(mktemp -d -p "$REPO/target")"
trap 'rm -rf "$work"' EXIT
export SCRIPT_DIR GEN OUT work

# Render one page, given its path under www/content. tola serves
# `<stem>.typ` at `/<stem>/` and `index.typ` at `/`, and the manifest must
# hold that route, so a page whose `<page-meta>` route differs fails here.
render_page() {
  local stem="${1%.typ}" route body err
  route="/${stem%index}"
  route="${route%/}/"
  if ! jq -e --arg r "$route" '.pages | has($r)' "$GEN/site-pages.json" >/dev/null; then
    echo "www/generate-md.sh: content/$1 is served at $route, which no page declares" >&2
    return 1
  fi
  body="$work/${stem//\//_}.md"
  err="$body.err"
  if ! typlite --root "$SCRIPT_DIR" --input markdown=1 \
      "$SCRIPT_DIR/content/$1" "$body" >"$err" 2>&1; then
    echo "www/generate-md.sh: typlite cannot render content/$1:" >&2
    cat "$err" >&2
    return 1
  fi
  # page.typ writes the title as an HTML h1, because typlite maps `=` to `##`.
  sed -i -E '1s|^<h1>([^<>]*)</h1>$|# \1|' "$body"
  # typlite falls back to HTML for a construct Markdown cannot hold, such as a
  # table cell with two paragraphs, and keeps an h1 whose title has markup.
  if ! awk -v f="content/$1" '
      /^```/ { fence = !fence; next }
      !fence && /^<\/?(h[1-6]|table|tr|td|div|span|p|ul|ol|li|figure|section|a|img|svg)[ >\/]/ {
        printf "www/generate-md.sh: %s line %d is HTML: %.80s\n", f, NR, $0
        bad = 1
      }
      END { exit bad }' "$body" >&2; then
    return 1
  fi
  mkdir -p "$OUT$route"
  mv "$body" "${OUT}${route}index.md"
}
export -f render_page

cd "$SCRIPT_DIR/content"
find . -name '*.typ' -printf '%P\0' | sort -z |
  xargs -0 -n1 -P "$(nproc)" bash -c 'render_page "$1"' _

# llms.txt. A page with no summary of its own, a command page, takes the
# command's one-line help from cli.json.
jq -r --slurpfile cli "$GEN/cli.json" '
  .url as $url
  | .pages as $p
  | ($cli[0].commands | map({key: "/cli/\(.name)/", value: .about}) | from_entries) as $about
  | def entry($r):
      "[\($p[$r].title)](\($url)\($r)index.md)"
      + (($p[$r].summary // $about[$r]) | if . then ": \(.)" else "" end);
  "# \($p["/"].title)",
  "",
  "> \($p["/"].summary)",
  "",
  "Each link below is the Markdown copy of a page on \($url)/. The page itself is the same URL without `index.md`.",
  (.groups[]
    | "", "## \(.title)", "",
      (.routes[] as $r
        | "- " + entry($r),
          ($p | keys_unsorted[]
            | select($r != "/" and . != $r and startswith($r))
            | "  - " + entry(.))))
' "$GEN/site-pages.json" >"$OUT/llms.txt"

# Every link to a Markdown copy, in a copy or in llms.txt, names a file this
# script wrote.
cd "$OUT"
dead="$(find . \( -name index.md -o -name llms.txt \) -print0 |
  { xargs -0 grep -oh "$SITE_URL/[^][()#\" ]*index\.md" || true; } | sort -u |
  while read -r link; do
    [ -f "$OUT${link#"$SITE_URL"}" ] || echo "  $link"
  done)"
if [ -n "$dead" ]; then
  echo "www/generate-md.sh: links to Markdown copies that do not exist:" >&2
  echo "$dead" >&2
  exit 1
fi
