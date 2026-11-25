#!/usr/bin/env bash
# Strip tola's hidden enhance stylesheet from the built pages, then hold the
# chrome's release version to [workspace.package].
#
# tola 0.7.1 injects a link to a hidden .tola/ stylesheet into every page. Its
# rules live at the end of assets/styles/main.css. A static capture never serves
# hidden paths. Run this after `tola build`. It is idempotent.
#
# The version arm reads the same `[workspace.package]` value site.typ's
# `mt-version` does, through www/generated/crate-map.json. The sidebar footer
# shows it on every page and each article rail adds a `Version` line.
# `pkg-version` is the Typst prelude's own, and a slot that renders it shows a
# version the release does not carry.

set -eo pipefail

www=$(cd "$(dirname "$0")" && pwd)
out=${1:-$www/public/mindtape}

if [ ! -d "$out" ]; then
  echo "post-build: $out not found" >&2
  exit 1
fi

find "$out" -name '*.html' -print0 |
  xargs -0 -r sed -i -E 's#<link rel="stylesheet" href="[^"]*/\.tola/enhance-[^"]*\.css" />##g'

rm -rf "$out/.tola"

version=$(sed -n 's/^version = "\([0-9][^"]*\)"/\1/p' "$www/../Cargo.toml" | head -1)
if [ -z "$version" ]; then
  echo "post-build: no [workspace.package] version in Cargo.toml" >&2
  exit 1
fi

no_footer=$(grep -rL -F "<span>v$version</span>" --include='*.html' "$out" || true)
if [ -n "$no_footer" ]; then
  echo "post-build: these pages lack v$version in the sidebar footer:" >&2
  printf '%s\n' "$no_footer" >&2
  exit 1
fi

no_rail=$(grep -rl -F 'class="page-rail"' --include='*.html' "$out" |
  xargs -r grep -L -F "<span>Version $version</span>" || true)
if [ -n "$no_rail" ]; then
  echo "post-build: these pages lack Version $version in the article rail:" >&2
  printf '%s\n' "$no_rail" >&2
  exit 1
fi

# The rail names the section and the version, and counts nothing: a page count
# comes back only if someone adds it to the rail's meta block again.
counted=$(grep -rlE '<span>[0-9]+ PAGES</span>' --include='*.html' "$out" || true)
if [ -n "$counted" ]; then
  echo "post-build: these pages count pages in the article rail:" >&2
  printf '%s\n' "$counted" >&2
  exit 1
fi
