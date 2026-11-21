#!/usr/bin/env bash
# Mirror the built asset trees to the repo root for a static page capture.
#
# The page links every asset as /mindtape/..., and the capture tool serves the
# project root. The capture then resolves those URLs at <root>/mindtape. It
# reads only the assets, never the other routes. This script regenerates the
# mirror, and git ignores it. Rerun it after a build. The capture manifest lists
# these mirrored paths.

set -eo pipefail

www=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$www/.." && pwd)
site=${1:-$www/public/mindtape}
mirror=$root/mindtape

rm -rf "$mirror"
mkdir -p "$mirror"
cp -r "$site/assets" "$mirror/assets"
cp -r "$site/pagefind" "$mirror/pagefind"
