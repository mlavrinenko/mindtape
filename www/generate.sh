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
#
# Docs SSG gotchas (tola), learned building this generator:
#   - Build-time images (the hero-card SVG, the terminal-ansi html.frame renders)
#     live only under generated/, never copied to public/. `tola validate`
#     statically flags an `image("<path>")` string literal as a broken link when
#     the target isn't a copied asset, so the pages inline these via
#     `image(read(path, encoding: none), format: "svg")` — validate ignores byte
#     reads (as it does the `.txt`/`.json` reads) and the inline is identical.
#     Keep any new build-time image on the read() path, not a bare `image(path)`.
#   - `tola serve` caches its file index at startup: a brand-new generated/ file
#     is not found by an already-running serve (pre-existing files read fine).
#     Restart serve (`just docs-serve`) after adding a capture here; `tola build`
#     (the gate path) always re-scans and is unaffected.
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

# The colour-bearing transcripts (the `mt ls` family) captured a SECOND time
# WITH ANSI, for the terminal-ansi html.frame render path (www/utils/ui.typ);
# the plain .txt above stays for the selectable-text terminals. `mt check` on a
# valid fixture and `views` emit no colour, so only `ls*` gets an .ansi.
gen_color() { local slug="$1"; shift; "$MT" -w "$FIX" --color always "$@" >"$OUT/$slug.ansi"; }
gen_color ls       ls
gen_color ls-open  ls 'status != "done"'

# A single task object for the for-agents page's `mt ls --json` example. The
# `ls` path scores priority and derives folder tags (unlike single-file `eval`,
# which leaves both empty); jq slices the envelope's `tasks` array to an
# object, so the page shows a complete, binary-sourced task instead of a
# hand-typed one. `mt ls --json`'s top-level shape is the `{tasks, applied}`
# envelope (`truth/tasks/conditional-ls-rules.typ`), captured separately below.
"$MT" -w "$FIX" --color never ls 'id == "fur-elise-polish"' --json \
  | jq '.tasks[0]' >"$OUT/ls-task.json"

# The full `mt ls --json` envelope (bare listing — [[ls.rules]] +
# default-view attribution) for the for-agents page.
gen_read ls-json ls --json

# config show --origin prints the project's config path absolute (canonicalized
# root). Normalize the fixture prefix to a project-relative path so the docs are
# machine-independent and reproducible; the rest is verbatim.
"$MT" -w "$FIX" --color never config show --origin \
  | sed "s#$FIX/##g" >"$OUT/config-origin.txt"

# The hero "source" face reads a real fixture task verbatim. Copy it under
# generated/ so it lives inside the Typst build root (read() can't escape it).
cp "$FIX/tasks/repertoire/fur-elise-polish.typ" "$OUT/hero-source.typ"

# The hero "typeset" face is that SAME task, genuinely compiled by the `task`
# show rule (lib/prelude.typ) — not hand-built HTML. Typst HTML export inlines
# the SVG as a self-contained data-uri `<img>`, so nothing lands in public/.
# Resolve @local/mindtape via a throwaway local-package path symlinked at lib/,
# so this never depends on `just install-lib` being run first.
PKGS="$(mktemp -d)"
LIBVER="$(sed -n 's/^version = "\(.*\)"/\1/p' "$REPO/lib/typst.toml" | head -1)"
mkdir -p "$PKGS/local/mindtape"
ln -s "$REPO/lib" "$PKGS/local/mindtape/$LIBVER"
typst compile --package-path "$PKGS" \
  "$FIX/tasks/repertoire/fur-elise-polish.typ" "$OUT/hero-card.svg"
rm -rf "$PKGS"

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
