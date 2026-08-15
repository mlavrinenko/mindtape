#!/usr/bin/env bash
# .just/scripts/fmt-fixture-check.sh — the body of the `fmt-fixture-check`
# recipe, extracted under truth/tasks/extract-the-long-gate-recipe-bodies-into-
# scripts.typ. Runs from the repo root, like the recipe that invokes it.
#
# Lock the formatter gates: synthesize a deliberately-mis-formatted TOML,
# Typst and just-module fixture in a temp dir, then assert `tombi format
# --check`, `typstyle --check` and `just --justfile <f> --fmt --check` all
# fail. The fmt recipe is what `just fmt`/`fmt-check` apply to the tracked
# config + docs + justfile tree, so the same formatters must reject the
# mis-formatted fixture (and accept a re-formatted one). A formatter that
# silently accepts drift would let `just fmt-check` pass on an out-of-sync
# editor save — exactly the bug this gate was added to prevent.
set -eo pipefail
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
bad_toml="$tmp/bad.toml"
bad_typ="$tmp/bad.typ"
bad_just="$tmp/bad.just"
printf '[a]\nb=1\nc  =   2\n' > "$bad_toml"
printf '#let   x   =   1\n#let y=2\n' > "$bad_typ"
printf 'set unstable := true\nfixture:\n  echo drifted\n' > "$bad_just"
if tombi format --check --offline "$bad_toml" >/dev/null 2>&1; then
    echo "fmt-fixture-check: tombi accepted the mis-formatted TOML — gate is broken" >&2
    exit 1
fi
if typstyle --check "$bad_typ" >/dev/null 2>&1; then
    echo "fmt-fixture-check: typstyle accepted the mis-formatted Typst — gate is broken" >&2
    exit 1
fi
# The just arm is spelled with the SAME `--justfile <file> --fmt --check`
# invocation `fmt-check` runs over JUST_FMT_PATHS, because the bug it locks is
# not "the formatter is broken" but "the invocation shape misses the file": a
# bare `just --fmt` skips module files entirely, so a fixture checked the wrong
# way would pass while the real gate saw nothing
# (truth/tasks/gate-formatting-of-just-module-files.typ). The fixture carries
# `set unstable := true` for the same reason every listed module file must —
# `--fmt` is refused without it, and a refusal is a non-zero exit that would
# pass this assertion for the wrong reason, so the arm asserts the SUCCESS case
# too: the same file, reformatted, must be accepted.
if just --justfile "$bad_just" --fmt --check >/dev/null 2>&1; then
    echo "fmt-fixture-check: just accepted the mis-formatted module file — gate is broken" >&2
    exit 1
fi
just --justfile "$bad_just" --fmt >/dev/null 2>&1
if ! just --justfile "$bad_just" --fmt --check >/dev/null 2>&1; then
    echo "fmt-fixture-check: just rejected its OWN formatting of the module file — the failure above was not about drift" >&2
    exit 1
fi
echo "fmt-fixture-check: all three formatters reject drift"
