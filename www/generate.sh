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
# `just docs::gen` invokes this; docs::build / docs::serve / docs::check run it first.
#
# One capture is not an `mt` transcript: release-targets.json reads
# .github/workflows/release.yml (the build-matrix targets and the
# `cargo publish -p` crate list) via `yq`, for the same reason as everything
# else here — the install page must derive from the pipeline that actually
# ships artifacts, never transcribe it by hand.
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
#     Restart serve (`just docs::serve`) after adding a capture here; `tola build`
#     (the gate path) always re-scans and is unaffected.
set -eo pipefail

# The fixed docs clock. The learn-piano fixture's dates are authored around it.
export MINDTAPE_NOW=2026-07-06

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
FIX="$REPO/examples/learn-piano"
OUT="$SCRIPT_DIR/generated"
MT="$REPO/target/debug/mt"

# Serialize every writer of www/generated/ (see www/generate-facts.sh for the
# other one). `just check` is [parallel] and three of its arms touch this
# directory: docs::check runs this script, while check-doc-facts and
# docs::md-check regenerate the facts inside it. The `rm -rf` below would then
# delete the facts out from under a concurrent reader — an intermittent
# "file not found (searched at www/generated/crate-map.json)" on work that is
# perfectly correct (truth/tasks/stop-the-generated-docs-writers-racing-under-
# parallel-check.typ). Re-exec under flock rather than locking at each call
# site, so a new caller is safe by default instead of by remembering.
#
# The lock lives under target/ because this script deletes the directory it
# would otherwise live in.
#
# The build runs BEFORE the lock is taken, deliberately. Every reader of
# www/generated/ now holds this lock for its whole run (docs/generate-md.sh
# and the check-doc-facts recipe both do), so anything held inside it is time
# the rest of `just check`'s [parallel] arms spend blocked. This is the DEBUG
# binary: the captures are --help/_cli/config/ls transcripts, byte-identical to
# release (nothing here depends on optimisation), and `target/debug/mt` is the
# artifact the test suite already keeps warm — so a docs run pays a ~15s debug
# rebuild, not a minutes-long fat-LTO relink (truth/tasks/capture-docs-output-
# with-the-debug-binary-not-release.typ).
# `--tests` keeps this the same `mt` the test arm links (the dev-dependency
# features), so it never relinks `target/debug/mt` under that arm; see
# docs/contributing/gates.md, "One mt binary, one feature set".
( cd "$REPO" && cargo build -p mindtape-cli --tests -q )

# Taken on a file descriptor rather than by re-exec'ing under `flock`, because
# a re-exec would rerun the build above. The lock is held until this script
# exits and fd 9 closes. MINDTAPE_GENERATED_LOCK tells the nested
# generate-facts.sh and generate-site-pages.sh calls at the end of this file
# that the lock is already held, so neither deadlocks waiting for a lock this
# process owns.
#
# Acquisition itself is conditional on that same variable, for a second
# reason beyond the nested calls: check-doc-facts.sh and docs/generate-md.sh
# both call THIS script now (truth/tasks/derive-site-pages-json-from-the-
# pages-page-meta-blocks.typ swapped their `bash www/generate-facts.sh` for
# `bash www/generate.sh`, since the site-pages manifest now needs this
# script's captures too), and both already hold the lock — via their own
# `exec 9>...; flock 9` — across the regenerate AND their own subsequent
# reads, for the same race this comment block opens with. Without this
# check, this script's unconditional `exec 9>...; flock 9` would open a
# SECOND file descriptor on the same lock file and block forever on the
# first one, held by a parent that is itself blocked waiting for this
# script to exit — a guaranteed deadlock. Skipping the acquisition when a
# caller already holds it is the same reentrant handshake generate-facts.sh
# has always used for the nested call below; this is that handshake applied
# one level up, now that generate.sh itself can be the nested call.
mkdir -p "$REPO/target"
if [ -z "$MINDTAPE_GENERATED_LOCK" ]; then
  exec 9>"$REPO/target/www-generated.lock"
  flock 9
  export MINDTAPE_GENERATED_LOCK=1
fi

rm -rf "$OUT"
mkdir -p "$OUT"

# A read command against the fixture in place → OUT/<slug>.txt (verbatim stdout).
gen_read() {
  local slug="$1"; shift
  "$MT" -w "$FIX" --color never "$@" >"$OUT/$slug.txt"
}

# --- version + structured command tree ------------------------------------
"$MT" --version >"$OUT/version.txt"
# The command tree as JSON (clap reflection via the hidden `_cli`). cli.typ
# renders it into the styled CLI reference — grouped commands and flag tables,
# not a verbatim `--help` dump. The `#[derive(Parser)]` structs are the single
# source, so the reference cannot restate or drift from a flag fact; the
# reference-only prose and worked examples live in www/utils/cli-notes.typ.
"$MT" _cli --json >"$OUT/cli.json"

# The config JSON Schema (project-independent — the exact bytes `mt install`
# also writes to disk). config.typ renders it into the generated Sections
# table; config-gate (www/utils/config-ref.typ) asserts every schema property
# is surfaced, so a newly added config key cannot silently vanish — same
# discipline as cli.json above.
"$MT" config schema >"$OUT/config-schema.json"

# --- release workflow: prebuilt targets + published crates ----------------
# Derived from .github/workflows/release.yml by generate-release-targets.sh
# (its own file, so this one clears the Shell cap); the call sits here, ahead
# of the read-command captures below, because the install page it feeds is not
# one of them.
bash "$SCRIPT_DIR/generate-release-targets.sh" "$REPO" "$OUT"

# --- read commands against the fixture (verbatim) -------------------------
gen_read ls        ls
gen_read ls-open   ls '!terminal'
gen_read ls-stuck  ls @stuck
gen_read ls-week   ls @this-week
gen_read views     views
gen_read check     check
gen_read describe  add --describe
gen_read flip-list flip --list
gen_read eval      eval "$FIX/tasks/repertoire/fur-elise-polish.typ"

# examples/self-validating/, the mindtape-tasks pack and examples/links/: each its own script (this file is at the Shell cap already).
bash "$SCRIPT_DIR/generate-self-validating.sh" "$REPO" "$OUT"; bash "$SCRIPT_DIR/generate-rule-packs.sh" "$REPO" "$OUT"; bash "$SCRIPT_DIR/generate-examples.sh" "$REPO" "$OUT"

# The colour-bearing transcripts (the `mt ls` family) captured a SECOND time
# WITH ANSI, for the terminal-ansi html.frame render path (www/utils/ui.typ);
# the plain .txt above stays for the selectable-text terminals. `mt check` on a
# valid fixture and `views` emit no colour, so only `ls*` gets an .ansi.
gen_color() { local slug="$1"; shift; "$MT" -w "$FIX" --color always "$@" >"$OUT/$slug.ansi"; }
gen_color ls       ls
gen_color ls-open  ls '!terminal'

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
# The copy has to preserve the fixture's DEPTH: its `[[typst.local-packages]]`
# entry resolves `@local/mindtape` at `../../lib` — this checkout's prelude
# rather than the machine-global one — so a copy parked one level down loads
# nothing, and every `mt` call below fails on the task it just wrote. Rebuild
# the two levels and symlink the real lib/ at the far end, the same throwaway
# resolution the hero card above builds for `typst compile`.
COPY_ROOT="$(mktemp -d)"
mkdir -p "$COPY_ROOT/examples"
ln -s "$REPO/lib" "$COPY_ROOT/lib"
COPY="$COPY_ROOT/examples/lp"
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

rm -rf "$COPY_ROOT"

# --- favicon + apple touch icon, rasterised from the one hand-maintained SVG --
# assets/images/ (unlike generated/) IS a tola-copied dir (see the header
# comment above), so a file written here lands at a real, stable public URL —
# the <link> tags in layout.typ point at it. Regenerated every build from
# logo.svg, the sole hand-maintained copy of the mark, so the raster can never
# drift from it; both PNGs are gitignored (see www/.gitignore / www/.ignore).
LOGO="$SCRIPT_DIR/assets/images/logo.svg"
rsvg-convert -w 32 -h 32 "$LOGO" -o "$SCRIPT_DIR/assets/images/favicon-32.png"
rsvg-convert -w 180 -h 180 "$LOGO" -o "$SCRIPT_DIR/assets/images/apple-touch-icon.png"

# Restore the derived facts inside the same critical section that deleted
# them. Splitting the wipe and the restore across two lock acquisitions would
# leave a window where www/generated/ exists with no facts in it, which is the
# exact state a concurrent reader must never see. `just docs::gen` still calls
# generate-facts.sh after this script — that run is idempotent and locked, and
# it keeps the recipe honest about what it produces.
bash "$SCRIPT_DIR/generate-facts.sh"

# Then the page manifest — after generate-facts.sh, never before, and inside
# the SAME lock: compiling a page (www/generate-site-pages.sh's query is a
# full compile) imports layout.typ -> site.typ, which needs crate-map.json
# (PKG_VERSION) just written above, plus every capture this script itself
# wrote (hero-source.typ, cli.json, …) — a page importing utils/ui.typ dies on
# a top-level read() otherwise (measured in the umbrella task). Its own file
# because this one is already at .linecop.yaml's 300-line Shell cap.
bash "$SCRIPT_DIR/generate-site-pages.sh"
