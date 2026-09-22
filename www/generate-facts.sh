#!/usr/bin/env bash
# www/generate-facts.sh — derive the drift-prone doc facts from their sources.
#
# The root docs restate a handful of facts that already have a machine-
# readable source: the crate map + dependency versions, the task DSL's core
# field list, and the `just check` gate table. Every one of those has rotted
# before (internals.typ said "three general-purpose crates" for a Workspace
# that ships seven; AGENTS.md still lists a `comfy-table` dependency that was
# removed). This script is the fix at the root: read the source once, write
# the fact to www/generated/ (gitignored, a build artifact like generate.sh's
# captures), and let a doc STATE the fact by reading the file, never by
# repeating it by hand.
#
# www/generate.sh calls this at its END, inside the flock it already holds:
# that script `rm -rf`s the whole generated/ dir on entry, and the wipe and
# the restore have to be ONE critical section or a concurrent reader sees the
# gap between them. `just docs::gen` calls it again — idempotent, and it keeps
# the recipe honest about what it produces. A caller that wants the page
# manifest too (check-doc-facts, docs/generate-md.sh) calls generate.sh rather
# than this script: www/generate-site-pages.sh derives that one and needs
# generate.sh's own captures — see both headers for why.
#
# Eight artifacts, eight sources:
#   - crate-map.json    <- `cargo metadata`, rust-toolchain.toml, the
#                          [workspace.package] table, lib/typst.toml
#   - dsl-fields.json   <- lib/prelude.typ's `task` signature, read back
#                          through its own `<mindtape>` marker
#   - frameworks.json   <- crates/mindtape/src/frameworks.toml plus the two
#                          scales in defaults.toml (see www/frameworks.jq)
#   - gates.json        <- `just --dump --dump-format json` for the
#                          `[group("gate")]` tags and `check`'s own dependency
#                          list, for row order (see www/gates.jq)
#   - just.typ          <- the same `just --dump` as gates.json (one
#                          shell-out, two artifacts): a Typst module, one
#                          function per public recipe
#   - linecop-caps.json <- .linecop.yaml, via yq
#   - sweep-flags.json  <- the same `just --dump` (see www/sweep-flags.jq)
#   - readme-task.json  <- the real backlog task README and the site root
#                          quote, trimmed to its marker (README_TASK_STEM)
#
# linecop-caps.json is the one artifact still without a consumer: it was
# derived for truth/tasks/generate-the-root-markdown-from-typst-with-typlite.typ
# beside gates.json, which docs/src/contributing.typ's gate table now reads.
#
# Deliberately out of scope: the design invariants (Typst eval-only, evalexpr
# walked not evaluated, the prelude-vs-Rust split). Those change with the
# design, not with a version bump — no source here proves or restates them.
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="$SCRIPT_DIR/generated"
mkdir -p "$OUT"

cd "$REPO"
mkdir -p target

# The flock re-exec guard and the atomic emit()/emit_raw() helpers — shared
# with the other www/generated/ writers, see www/facts-lib.sh.
source "$SCRIPT_DIR/facts-lib.sh"

# --- crate-map.json ---------------------------------------------------------
# The derivation itself lives in www/crate-map.jq (argued in place there,
# alongside its gates.jq and sweep-flags.jq siblings); what stays here is the
# metadata call and the handful of facts that jq program cannot read for
# itself.
META="$(cargo metadata --format-version 1)"

# WHICH crates are substrate is derived too, not listed: they are exactly the
# packages whose own manifest points at this author's GitHub org and that are
# not workspace members — i.e. the crates extracted OUT of this repo and now
# consumed back from crates.io. Verified exact against today's graph (seven
# hits, no false positives; `mindtape`/`mindtape-cli` share the org and are
# excluded as workspace members). A hand-written list was the last hardcoded
# claim in this file and would go stale the day an eighth crate is extracted —
# which is the precise failure this whole script exists to prevent. When one
# is, it appears here automatically and `just check-doc-facts` fails until it
# has prose, instead of silently going unmentioned.
SUBSTRATE_REPO_RE='github\.com/mlavrinenko/'

# The architecture-level dependencies the docs name by hand. This one IS a
# list, and has to be: "which deps are worth naming in an architecture
# sentence" is an editorial judgement with no machine source. Only the
# VERSIONS are derived. A dep that leaves the graph resolves to null rather
# than silently vanishing, so a stale editorial claim surfaces as a visible
# hole (this is how `comfy-table` — replaced by the hand-rolled renderer in
# crates/mindtape-cli/src/render/ — was caught).
ARCH_DEPS='["typst","typst-eval","evalexpr","clap","serde","toml","serde_json","csv"]'

CHANNEL="$(sed -n 's/^channel = "\(.*\)"/\1/p' rust-toolchain.toml | head -1)"
WS_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
WS_EDITION="$(sed -n 's/^edition = "\(.*\)"/\1/p' Cargo.toml | head -1)"
WS_RUST_VERSION="$(sed -n 's/^rust-version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
TYPST_LIB_VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' lib/typst.toml | head -1)"

jq -f "$SCRIPT_DIR/crate-map.jq" \
   --arg substrate_re "$SUBSTRATE_REPO_RE" \
   --argjson archdeps "$ARCH_DEPS" \
   --arg channel "$CHANNEL" \
   --arg ws_version "$WS_VERSION" \
   --arg ws_edition "$WS_EDITION" \
   --arg ws_rust_version "$WS_RUST_VERSION" \
   --arg typst_lib_version "$TYPST_LIB_VERSION" \
   <<<"$META" | emit crate-map.json

# --- dsl-fields.json ---------------------------------------------------------
# lib/prelude.typ's `task` function names every core field as a parameter, and
# emits their names+defaults verbatim into the `<mindtape>` marker's `fields`
# dict — so the field list is read back off that marker rather than mirrored
# by a regex over the signature (a regex drifts the moment the signature's
# formatting does; this doesn't, because it evaluates the same code the marker
# comes from). Importing by path (`/lib/prelude.typ`), not `@local/mindtape`,
# means this never depends on `just install-lib` having run. The driver lives
# under target/ because `typst query --root .` refuses a source outside the
# project root — same trick `just check-doc-coverage` already uses.
tmp="$(mktemp -d -p target)"
trap 'rm -rf "$tmp"' EXIT
printf '#import "/lib/prelude.typ": task\n#show: task.with()\n= body\n' \
  >"$tmp/dsl-driver.typ"
fields_dict="$(typst query --root . "$tmp/dsl-driver.typ" '<mindtape>' \
  --field value --one | jq '.fields')"
jq -n --argjson fields "$fields_dict" \
  '{fields: [$fields | keys_unsorted[] as $k | {name: $k, default: $fields[$k]}]}' \
  | emit dsl-fields.json
rm -rf "$tmp"

# --- frameworks.json ---------------------------------------------------------
# The baked-in priority frameworks and the two scales they land on, off the
# TOML that declares them: names, typed inputs, score formulas, category
# labels, bands and anchors. `tomlq` is the same yq package linecop-caps.json
# already uses below, pointed at TOML instead of YAML. Argued in
# www/frameworks.jq; the scales ride in as an argument because they live in a
# different file from the frameworks that score onto them.
SCALES="$(tomlq -c '.tasks | {priority, difficulty}' crates/mindtape/src/defaults.toml)"
tomlq -f "$SCRIPT_DIR/frameworks.jq" --argjson scales "$SCALES" \
  crates/mindtape/src/frameworks.toml | emit frameworks.json

# site-pages.json — the docs site's page manifest — used to be derived here,
# read back off www/utils/site.typ's PAGES dict. It moved to its own script,
# www/generate-site-pages.sh, under truth/tasks/derive-site-pages-json-from-
# the-pages-page-meta-blocks.typ: each page carries its OWN `<page-meta>` block
# now, so the manifest comes from the pages themselves and deriving it needs a
# full page compile — www/generate.sh's captures, not just this script's. The
# header above carries what that cost this script's callers.

# --- gates.json --------------------------------------------------------------
# Gate list + row order from the Justfile's `[group("gate")]` tags and
# `check`'s own dependency list (also the cross-check the two are read against
# each other for) — see www/gates.jq's header for why row order reads that list
# rather than `just --summary --unsorted`'s source order now that a gate can
# live in a module. Each gate carries a derived `command`, and the artifact as
# a whole carries the fmt path assignments CONTRIBUTING.md renders verbatim
# (selected in www/gates.jq by their shared `_FMT_PATHS` suffix, so a future
# formatter's list lands without an edit). The jq program is long enough to
# blow this script's own linecop cap, so it lives in www/gates.jq.
JUST_DUMP="$(just --dump --dump-format json)"
jq -f "$SCRIPT_DIR/gates.jq" <<<"$JUST_DUMP" | emit gates.json

# --- just.typ ------------------------------------------------------------
# Reuses $JUST_DUMP above — one `just --dump` shell-out, two artifacts. The
# module and its functions are explained in JUST_TYP_HEADER below, which
# ships verbatim inside the artifact itself so that explanation travels with
# the file. What is NOT in the shipped header: the identifier collision
# check — two recipes mangling to the same `-`-joined identifier fail the
# jq program below with halt_error, naming both, rather than one silently
# shadowing the other.
#
# A module recipe (truth/tasks/split-the-justfile-into-modules.typ) is walked
# via the same recursive all_recipes() the gates.jq program uses, keyed by its
# `namepath` (just's own dump already qualifies it, e.g. "docs::check"). The
# rendered COMMAND swaps `::` for a space — `just docs check`, matching how
# `just --list --list-submodules` itself presents a module recipe — while the
# Typst IDENTIFIER swaps `::` for `-`, so `docs::check` reads `#just.docs-check()`
# and mangles right back to the pre-split flat name. That is what lets a
# recipe move into a module without touching a line of doc prose.
JUST_TYP_HEADER="$(cat <<'HDR'
// www/generated/just.typ — generated by www/generate-facts.sh from `just
// --dump --dump-format json`; do not edit, edit the Justfile.
//
// A Typst MODULE, not a dict — Typst reads a dict field call as a method
// call and errors "type dictionary has no method" (verified). One function
// per PUBLIC recipe (private ones, `_name`, are excluded from the docs
// surface, same as check-doc-coverage's `hide = true` exclusion), module
// recipes included. Each renders the invocation as a raw span:
// `#just.check()` -> `just check`, `#just.docs-check()` -> `just docs check`;
// extra positional args join onto the command, so
// `#just.mutants("-f <file>")` -> `just mutants -f <file>`.
//
// Import as a module: `#import "/www/generated/just.typ" as just`.
// docs/src/lib.typ re-exports it so a docs/src source imports one place;
// www/content/*.typ (a different render path) imports this file directly.
//
// Identifier = the invocation path with `::` mangled to `-` (see this
// file's own producer, www/generate-facts.sh, for why).
#let mk(name) = (..args) => raw(
  "just " + name + args.pos().map(a => " " + a).join(""),
)
HDR
)"
jq -r --arg header "$JUST_TYP_HEADER" '
  def all_recipes:
    ((.recipes // {}) | to_entries | map(.value))
    + ((.modules // {}) | to_entries | map(.value) | map(all_recipes) | add // []);

  [all_recipes[] | select(.private == false)
    | {name: (.namepath | gsub("::"; " ")), id: (.namepath | gsub("::"; "-"))}] as $recipes
  | ($recipes | group_by(.id) | map(select(length > 1))) as $dups
  | if ($dups | length) > 0 then
      ("just.typ: two recipes mangle to the same identifier:\n"
        + ($dups | map("  " + .[0].id + ": " + (map(.name) | join(", ")))
           | join("\n")))
      | halt_error(1)
    else
      $header + "\n\n"
        + ($recipes | sort_by(.id)
            | map("#let " + .id + " = mk(\"" + .name + "\")")
            | join("\n"))
        + "\n"
    end
' <<<"$JUST_DUMP" | emit_raw just.typ

# --- sweep-flags.json ---------------------------------------------------------
# `ask-refactoring`'s five `[arg(...)]` parameters, off the same $JUST_DUMP —
# the signal sweep's thresholds and their defaults, in one place, rendered by
# docs/contributing/signal-sweep.md. Argued in www/sweep-flags.jq.
jq -f "$SCRIPT_DIR/sweep-flags.jq" <<<"$JUST_DUMP" | emit sweep-flags.json

# --- linecop-caps.json --------------------------------------------------------
# The exact `limits`/`overrides` .linecop.yaml carries — the source CONTRIBUTING.md
# quotes caps from. yq is the same jq-over-YAML wrapper generate.sh uses for the
# release workflow, so this stays consistent with the rest of the docs tooling.
yq '{limits, overrides}' .linecop.yaml | emit linecop-caps.json

# --- readme-task.json ---------------------------------------------------------
# The one real, closed backlog task README.md's and the site root's opening
# both quote, derived here once so both pages read the same marker and stem
# — www/content/index.typ cannot `read()` truth/tasks/ itself since tola's
# root is www/ and Typst refuses a path escaping the declared root.
# readme-task.yaml (the twin) has no real file to derive from; the grep loop
# below is its drift gate, failing by name if the marker stops matching it.
README_TASK_STEM="move-goal-verdicts-into-x-goals"
README_TASK_FILE="truth/tasks/$README_TASK_STEM.typ"
grep -q '^== ' "$README_TASK_FILE" || {
  echo "generate-facts.sh: $README_TASK_FILE missing, or has no body heading to trim after the marker" >&2
  exit 1
}
README_TASK_MARKER="$(awk '/^== /{exit} {print}' "$README_TASK_FILE")"
[[ "$README_TASK_MARKER" == *"check-goals is the only place they're checked now"* ]] || {
  echo "generate-facts.sh: $README_TASK_FILE's closing note changed — update readme-task.yaml to match" >&2
  exit 1
}
README_TASK_YAML="$(cat "$SCRIPT_DIR/readme-task.yaml")"
jq -n --arg stem "$README_TASK_STEM" --arg marker "$README_TASK_MARKER" --arg yaml "$README_TASK_YAML" \
  '{stem: $stem, marker: $marker, yaml: $yaml}' | emit readme-task.json
