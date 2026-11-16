#!/usr/bin/env bash
# www/generate-facts.sh: derive the drift-prone doc facts from their sources.
#
# The root docs restate facts that already have a machine-readable source:
# the crate map + dependency versions, the task DSL's core field list, the
# `just check` gate table. Each has rotted before. internals.typ said "three
# general-purpose crates" about a Workspace that holds seven, and AGENTS.md
# still listed a `comfy-table` dependency the repo had removed. This script
# is the fix at the root: read the source once, write the fact to
# www/generated/ (gitignored, a build artifact like generate.sh's
# captures). A doc states the fact by reading the file, never by repeating
# it by hand.
#
# www/generate.sh calls this at its end, inside the flock it already holds.
# That script `rm -rf`s the whole generated/ dir on entry. The wipe and the
# restore have to be one critical section, or a concurrent reader sees the
# gap between them. `just docs::gen` calls it again (idempotent), and it
# keeps the recipe honest about what it produces. A caller that wants the
# page manifest too (check-doc-facts, docs/generate-md.sh) calls generate.sh
# rather than this script. www/generate-site-pages.sh derives that one and
# needs generate.sh's own captures. See both headers for why.
#
# Twelve artifacts, twelve sources:
#   - crate-map.json    <- `cargo metadata`, rust-toolchain.toml, the
#                          [workspace.package] table, lib/typst.toml
#   - dsl-fields.json   <- lib/prelude.typ's `task` signature, read back
#                          through its own `<mindtape>` marker, and its
#                          status constructors
#   - frameworks.json   <- crates/mindtape/src/framework/frameworks.toml plus the two
#                          scales in defaults.toml (see www/frameworks.jq)
#   - date-tokens.json <- the `{date:FMT}` token table, read back from the
#                          `date_tokens` key of cli.json (`mt _cli`)
#   - filter-functions.json <- the filter registry, read back from the
#                          `filter_functions` key of cli.json (`mt _cli`)
#   - slot-kinds.json   <- the template slot kinds, read back from the
#                          `slot_kinds` key of cli.json (`mt _cli`)
#   - filter-fields.json <- the filter field registry, read back from the
#                          `fields` key of cli.json (`mt _cli`)
#   - link-kinds.json   <- the built-in link kinds and their rules, read back
#                          from the `link_kinds` key of cli.json (`mt _cli`)
#   - set-commands.json <- the commands that run on a set, read back from
#                          each leaf's `runs_on_set` in cli.json (`mt _cli`)
#   - gates.json        <- `just --dump --dump-format json` for the
#                          `[group("gate")]` tags and `check`'s own dependency
#                          list, for row order (see www/gates.jq)
#   - just.typ          <- the same `just --dump` as gates.json (one
#                          shell-out, two artifacts): a Typst module, one
#                          function per public recipe
#   - linecop-caps.json <- .config/linecop.yaml, via yq
#   - sweep-flags.json  <- the same `just --dump` (see www/sweep-flags.jq)
#
# linecop-caps.json is the one artifact without a consumer. This script
# derives it beside gates.json, which docs/src/contributing.typ's gate table reads.
#
# Deliberately out of scope: the design invariants (Typst eval-only, evalexpr
# walked not evaluated, the prelude-vs-Rust split). Those change with the
# design, not with a version bump, and no source here proves or restates them.
set -eo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="$SCRIPT_DIR/generated"
mkdir -p "$OUT"

cd "$REPO"
mkdir -p target

# The flock re-exec guard and the atomic emit()/emit_raw() helpers, shared
# with the other www/generated/ writers (see www/facts-lib.sh).
source "$SCRIPT_DIR/facts-lib.sh"

# crate-map.json
# The derivation itself lives in www/crate-map.jq, argued in place there
# alongside its gates.jq and sweep-flags.jq siblings. This file keeps the
# metadata call and the handful of facts that jq program cannot read for
# itself.
META="$(cargo metadata --format-version 1)"

# This script derives the substrate set too, rather than listing it. A
# substrate crate is exactly a package whose own manifest points at this
# author's GitHub org and that is not a workspace member. Those are the
# crates extracted out of this repo and consumed back from crates.io.
# Verified exact against today's graph: seven hits, no false positives.
# `mindtape`/`mindtape-cli` share the org, but workspace membership excludes
# them. A hand-written list was the last hardcoded claim here. It would go
# stale the day the repo extracts an eighth crate, the precise failure this
# whole script exists to prevent. When that happens, the crate appears here
# automatically and `just docs::facts` fails until it has prose, instead of
# silently going unmentioned.
SUBSTRATE_REPO_RE='github\.com/mlavrinenko/'

# The architecture-level dependencies the docs name by hand. This one is a
# list, and has to be: "which deps are worth naming in an architecture
# sentence" is an editorial judgement with no machine source. This script
# derives only the versions. A dep that leaves the graph resolves to null
# rather than silently vanishing. A stale editorial claim then surfaces as a
# visible hole. The removal of `comfy-table`, which the hand-rolled renderer
# in crates/mindtape-cli/src/render/ replaced, surfaced that way.
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

# dsl-fields.json
# lib/prelude.typ's `task` function names every core field as a parameter, and
# emits their names+defaults verbatim into the `<mindtape>` marker's `fields`
# dict. The field list comes back off that marker, not out of a regex over the
# signature. A regex drifts the moment the signature's formatting does. This
# doesn't, because it evaluates the same code the marker comes from. Importing
# by path (`/lib/prelude.typ`), not `@local/mindtape`, means this never depends
# on `just install::lib` having run. The driver lives under target/ because
# `typst query --root .` refuses a source outside the project root, the same
# trick `just docs::coverage` already uses.
tmp="$(mktemp -d -p target)"
trap 'rm -rf "$tmp"' EXIT
printf '#import "/lib/prelude.typ": task\n#show: task.with()\n= body\n' \
  >"$tmp/dsl-driver.typ"
fields_dict="$(typst query --root . "$tmp/dsl-driver.typ" '<mindtape>' \
  --field value --one | jq '.fields')"
# Each status constructor is a `#let <name> = _mk("<name>")` line, and an
# empty list fails the build, so a reformatted prelude cannot hide them.
statuses="$(sed -n 's/^#let \([a-z-]*\) = _mk("\1")$/\1/p' lib/prelude.typ \
  | jq -R . | jq -s .)"
[ "$statuses" != "[]" ] || {
  echo "generate-facts.sh: no status constructors found in lib/prelude.typ" >&2
  exit 1
}
jq -n --argjson fields "$fields_dict" --argjson statuses "$statuses" \
  '{fields: [$fields | keys_unsorted[] as $k | {name: $k, default: $fields[$k]}],
    statuses: $statuses}' \
  | emit dsl-fields.json
rm -rf "$tmp"

# frameworks.json
# The baked-in priority frameworks and their two scales, off the TOML that
# declares them: names, typed inputs, score formulas, category labels, bands
# and anchors. `tomlq` is the same yq package linecop-caps.json uses below,
# pointed at TOML instead of YAML. Argued in www/frameworks.jq. The scales
# ride in as an argument because they live in a different file from the
# frameworks that score onto them.
SCALES="$(tomlq -c '.tasks | {priority, difficulty}' crates/mindtape/src/config/defaults.toml)"
tomlq -f "$SCRIPT_DIR/frameworks.jq" --argjson scales "$SCALES" \
  crates/mindtape/src/framework/frameworks.toml | emit frameworks.json

# filter-functions.json
# The filter language's predicates and functions with their arities, off the
# registry in crates/mindtape/src/filter/eval.rs that dispatches them. cli.json
# is `mt _cli --json`, which www/generate.sh writes before this script runs.
jq '{functions: .filter_functions}' "$OUT/cli.json" | emit filter-functions.json

# date-tokens.json
# The `{date:FMT}` tokens with their descriptions, off the DATE_TOKENS table in
# crates/mindtape/src/date/mod.rs that `format_iso` matches against. The
# config page doesn't hold a token by hand.
jq '{tokens: .date_tokens}' "$OUT/cli.json" | emit date-tokens.json

# slot-kinds.json
# The kinds a template's `slot(kind: ...)` accepts, off `SlotKind::ALL` in
# crates/mindtape/src/template/mod.rs that `parse` and the refusals read.
# The templates page doesn't hold a kind by hand.
jq '{kinds: .slot_kinds}' "$OUT/cli.json" | emit slot-kinds.json

# filter-fields.json
# The fields a filter compares, each marked sortable, off `FIELDS` in
# crates/mindtape/src/model/fields.rs and `is_sort_field` in sort.rs. The
# filters page doesn't hold a field name or sort exception by hand.
jq '{fields: .fields}' "$OUT/cli.json" | emit filter-fields.json

# link-kinds.json
# The built-in link kinds, each with its `max`, `blocking` and `acyclic` rule,
# off `BUILTIN_KINDS` in crates/mindtape/src/link/mod.rs. The links page
# doesn't hold a kind name or count by hand.
jq '{kinds: .link_kinds}' "$OUT/cli.json" | emit link-kinds.json

# set-commands.json
# The flattened path of every leaf command marked `runs_on_set`, in `mt --help`
# order, off `CLASSIFICATION` in crates/mindtape-cli/src/cli/command_class.rs,
# the mark `main` refuses a set by. The sets page and the skill list these
# commands and hold none by hand.
jq '
  def leaves($prefix):
    ($prefix + [.name]) as $path
    | if (.subcommands | length) == 0
      then select(.runs_on_set) | $path | join(" ")
      else .subcommands[] | leaves($path)
      end;
  {commands: [.commands[] | leaves([])]}
' "$OUT/cli.json" | emit set-commands.json

# site-pages.json, the docs site's page manifest, comes from its own script,
# www/generate-site-pages.sh. Each page carries its own `<page-meta>` block.
# The manifest derives from the pages themselves, and deriving it needs a full
# page compile (www/generate.sh's captures, not just this script's). The
# header above carries what that costs this script's callers.

# gates.json
# Gate list + row order from the Justfile's `[group("gate")]` tags and
# `check`'s dependency list, which this script reads against each other as a
# cross-check. See www/gates.jq's header for why row order reads that list
# rather than `just --summary --unsorted`'s source order, since a gate can
# live in a module. The artifact also carries the fmt path assignments
# docs/contributing/gates.md renders verbatim. www/gates.jq selects them by
# their shared `_FMT_PATHS` suffix, so a future formatter's list lands without
# an edit. The jq program is long enough to blow this script's own linecop
# cap, so it lives in www/gates.jq.
JUST_DUMP="$(just --dump --dump-format json)"
jq -f "$SCRIPT_DIR/gates.jq" <<<"$JUST_DUMP" | emit gates.json

# just.typ
# Reuses $JUST_DUMP above: one `just --dump` shell-out, two artifacts.
# JUST_TYP_HEADER below explains the module and its functions. The artifact
# itself carries that header verbatim, and the explanation travels with the
# file. The header doesn't carry the identifier collision check. Two recipes
# mangling to the same `-`-joined identifier fail the jq program below with
# `halt_error`, naming both, rather than one silently shadowing the other.
#
# The same recursive all_recipes() the gates.jq program uses walks a module
# recipe (truth/tasks/2026-08/split-the-justfile-into-modules.typ), keyed by
# its `namepath`. just's own dump already qualifies it ("docs::check"). The
# rendered command swaps `::` for a space (`just docs check`), matching how
# `just --list --list-submodules` presents it. The Typst identifier swaps
# `::` for `-`, so `docs::check` reads `#just.docs-check()` and mangles
# right back to the pre-split flat name. That is what lets a recipe move
# into a module without touching a line of doc prose.
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
// `#just.mutants-sweep("-f <file>")` -> `just mutants::sweep -f <file>`.
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

# sweep-flags.json
# `ask-refactoring`'s six `[arg(...)]` parameters, off the same $JUST_DUMP (the
# signal sweep's thresholds and their defaults, in one place), rendered by
# docs/contributing/signal-sweep.md. Argued in www/sweep-flags.jq.
jq -f "$SCRIPT_DIR/sweep-flags.jq" <<<"$JUST_DUMP" | emit sweep-flags.json

# linecop-caps.json
# The exact `limits`/`overrides` .config/linecop.yaml carries (the source
# CONTRIBUTING.md quotes caps from). yq is the same jq-over-YAML wrapper generate.sh uses for the
# release workflow, so this stays consistent with the rest of the docs tooling.
yq '{limits, overrides}' .config/linecop.yaml | emit linecop-caps.json
