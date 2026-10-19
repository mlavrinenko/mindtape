#!/usr/bin/env bash
# Body of `just outdatty check`. `outdatty check` passes with a lock group that
# .config/outdatty.yaml has dropped, which a split or a rename leaves behind,
# so the second half catches it. Drop it once `outdatty check` reports an undeclared
# lock group upstream.
set -eo pipefail
outdatty check
declared=$(yq -r '.groups[].name' .config/outdatty.yaml | sort)
orphans=$(comm -13 <(echo "$declared") <(yq -r '.groups | keys[]' .config/outdatty.lock | sort))
[ -z "$orphans" ] || { echo ".config/outdatty.lock holds groups .config/outdatty.yaml does not declare, delete them: $orphans" >&2; exit 1; }
