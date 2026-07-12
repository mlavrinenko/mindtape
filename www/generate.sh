#!/usr/bin/env bash
# www/generate.sh — capture live `mt` output for the docs site.
#
# Runs the REAL `mt` binary against the examples/learn-piano fixture and writes
# each capture VERBATIM (plain text, `--color never`) under www/generated/. The
# docs pages inject these via `terminal(read("../generated/<slug>.txt"))`, so a
# transcript can never drift from the binary — regenerated on every build.
#
# Determinism: MINDTAPE_NOW pins "today" (the fixture authors its dates around
# it), so the same commit renders the same site build-to-build. The two captures
# that would leak an absolute machine path (`config show --origin`, `add`'s
# "wrote <path>") are path-normalized to a project-relative form — the only
# post-processing; the transcript content is otherwise the binary's stdout.
#
# `just docs-gen` invokes this; docs-build / docs-serve / docs-check run it first.
set -eo pipefail

# The fixed docs clock. The learn-piano fixture's dates are authored around it.
export MINDTAPE_NOW=2026-07-06

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
FIX="$REPO/examples/learn-piano"
OUT="$SCRIPT_DIR/generated"
MT="$REPO/target/release/mt"

# Build the binary the captures run against.
( cd "$REPO" && cargo build --release -p mindtape-cli )

rm -rf "$OUT"
mkdir -p "$OUT"

# A read command against the fixture in place → OUT/<slug>.txt (verbatim stdout).
gen_read() {
  local slug="$1"; shift
  "$MT" -w "$FIX" --color never "$@" >"$OUT/$slug.txt"
}

# Per-subcommand help (project-independent) → OUT/help-<cmd>.txt (verbatim).
gen_help() {
  local cmd="$1"
  "$MT" "$cmd" --help >"$OUT/help-$cmd.txt"
}

# --- version + global/per-command help (verbatim) -------------------------
"$MT" --version >"$OUT/version.txt"
"$MT" --help >"$OUT/help.txt"
for cmd in add ls views flip check config rm init install eval go review worktrees recur; do
  gen_help "$cmd"
done

# --- read commands against the fixture (verbatim) -------------------------
gen_read ls        ls
gen_read ls-open   ls 'status != "done"'
gen_read ls-stuck  ls @stuck
gen_read ls-week   ls @this-week
gen_read views     views
gen_read check     check
gen_read describe  add --describe
gen_read flip-list flip --list
gen_read eval      eval "$FIX/tasks/repertoire/fur-elise-polish.typ"

# config show --origin prints the project's config path absolute (canonicalized
# root). Normalize the fixture prefix to a project-relative path so the docs are
# machine-independent and reproducible; the rest is verbatim.
"$MT" -w "$FIX" --color never config show --origin \
  | sed "s#$FIX/##g" >"$OUT/config-origin.txt"

# The hero "source" face reads a real fixture task verbatim. Copy it under
# generated/ so it lives inside the Typst build root (read() can't escape it).
cp "$FIX/tasks/repertoire/fur-elise-polish.typ" "$OUT/hero-source.typ"

# The fixture's `mt add` template, verbatim — the docs' "A template" example, so
# it matches the `add --describe` slot schema captured above. Same root reason.
cp "$FIX/.mindtape/templates/task.typ" "$OUT/template.typ"

# --- mutating commands against a throwaway copy (never touch the fixture) --
COPY="$(mktemp -d)/lp"
cp -r "$FIX" "$COPY"

# add: file a new task; normalize the "wrote <abs>" path to project-relative.
"$MT" -w "$COPY" --color never add \
  --set title="Practice pedaling technique" \
  --set summary="Half-pedal control on sustained chords." \
  --set impact=6 --set ease=5 \
  | sed "s#$COPY/##g" >"$OUT/add.txt"
# The written task, verbatim, for the "no template machinery survives" example.
cp "$COPY/tasks/practice-pedaling-technique.typ" "$OUT/add-written.typ"

# done: close a wip task (pure status flip).
"$MT" -w "$COPY" --color never done fur-elise-polish >"$OUT/done.txt"

rm -rf "$(dirname "$COPY")"

echo "generated $(find "$OUT" -type f | wc -l) files under $OUT"
