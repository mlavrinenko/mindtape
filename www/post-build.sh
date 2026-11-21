#!/usr/bin/env bash
# Strip tola's hidden enhance stylesheet from the built pages.
#
# tola 0.7.1 injects a link to a hidden .tola/ stylesheet into every page. Its
# rules live at the end of assets/styles/main.css. A static capture never serves
# hidden paths. Run this after `tola build`. It is idempotent.

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
