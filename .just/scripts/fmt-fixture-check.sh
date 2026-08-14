#!/usr/bin/env bash
# .just/scripts/fmt-fixture-check.sh — the body of the `fmt-fixture-check`
# recipe, extracted under truth/tasks/extract-the-long-gate-recipe-bodies-into-
# scripts.typ. Runs from the repo root, like the recipe that invokes it; the
# prose arguing why the just arm is spelled `--justfile <file> --fmt --check`
# and why the SUCCESS case is asserted too stays above that recipe in the
# Justfile.
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
