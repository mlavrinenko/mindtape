#!/usr/bin/env bash
# .just/scripts/check-doc-blocks.sh: the body of `just docs::blocks`. Runs from the repo root.
#
# A code block typed by hand into a doc source shows text that nothing runs or
# validates, so it drifts from what mt accepts and the reader cannot tell. A
# block in www/content, www/utils or docs/src comes from a checked fixture, a
# capture read from www/generated/, or a derived fact. A checked fixture is a
# `docs:slice` that www/generate-examples.sh cuts from examples/. This gate
# fails, naming file and line, on a block typed straight into the source
# instead. _doc-blocks-lib.sh defines what counts as written in the source.
#
# A block a fixture cannot hold, such as a grammar sketch, carries
# `// docs:hand-typed <reason>` on the line above it. A marker without a
# reason, or with no block on the next line, fails.
#
# `PENDING` names the sources not yet converted. A listed file may keep its hand-typed
# blocks. A listed file with none left fails, and the list only shrinks. Each page's own
# task removes its files.
#
# The control (_doc-blocks-control.sh) runs first: a scanner with nothing to read would
# report a clean tree.
set -eo pipefail

# shellcheck source=.just/scripts/_doc-blocks-lib.sh
source .just/scripts/_doc-blocks-lib.sh

bash .just/scripts/_doc-blocks-control.sh

PENDING=(
    docs/src/contributing/generated-docs.typ
)

shopt -s globstar nullglob
tmp=$(mktemp)
trap 'rm -f "$tmp"' EXIT
doc_blocks_scan docs/src/**/*.typ www/content/**/*.typ www/utils/*.typ >"$tmp"
doc_blocks_judge "$tmp" "${PENDING[@]}"
