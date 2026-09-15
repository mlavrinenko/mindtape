#!/usr/bin/env bash
# www/generate-self-validating.sh — capture examples/self-validating/, the
# deliberately invalid fixture www/content/self-validating-tasks.typ quotes:
# a task's own x-goals field, checked by the project's own Typst function
# (lib.typ#check-goals), refusing a task that skips a goal.
#
# Split out of www/generate.sh (which calls this, still inside the lock it
# already holds) under the same 300-line Shell cap that split off
# generate-site-pages.sh — see that script's header for the reasoning this
# one does not repeat. Unlike that script, this one is never called
# standalone, so it takes no lock of its own.
#
# NOT in `.mmz/config.yaml`'s `examples` scope or the `check-examples`
# recipe: both validate every project they list, and this fixture is built
# to fail one rule on purpose. `mt check` against it exits non-zero, which
# would abort the caller's `set -eo pipefail` if run unguarded — caught here
# instead, with the code recorded beside the transcript rather than inside
# it, since `--color never` output is exactly what `mt` printed, nothing
# appended.
set -eo pipefail

REPO="$1"
OUT="$2"
MT="$REPO/target/debug/mt"
SV="$REPO/examples/self-validating"

sv_exit=0
"$MT" -w "$SV" --color never check >"$OUT/self-validating-check.txt" 2>&1 \
  || sv_exit=$?
printf '%s' "$sv_exit" >"$OUT/self-validating-check-exit.txt"

# The fixture's own source, copied verbatim so the page can read() them —
# Typst's HTML build runs under --root www and refuses a read escaping it,
# the same reason generate.sh copies hero-source.typ/template.typ out of
# examples/learn-piano instead of reading them in place.
cp "$SV/.mindtape/config.toml"               "$OUT/self-validating-config.toml"
cp "$SV/goals.typ"                           "$OUT/self-validating-goals.typ"
cp "$SV/lib.typ"                             "$OUT/self-validating-lib.typ"
cp "$SV/tasks/ship-onboarding-checklist.typ" "$OUT/self-validating-task-pass.typ"
cp "$SV/tasks/export-report-as-csv.typ"      "$OUT/self-validating-task-fail.typ"
