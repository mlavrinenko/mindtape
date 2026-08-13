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
# `just docs-gen` runs this after www/generate.sh (never before — that script
# `rm -rf`s the whole generated/ dir on entry, so anything written here first
# would be deleted). `just check-doc-facts` also calls this directly, so the
# gate is self-contained and never depends on a prior docs-gen run.
#
# Four artifacts, four sources:
#   - crate-map.json    <- `cargo metadata`, rust-toolchain.toml, the
#                          [workspace.package] table, lib/typst.toml
#   - dsl-fields.json   <- lib/prelude.typ's `task` signature, read back
#                          through its own `<mindtape>` marker
#   - gates.json        <- `just --dump --dump-format json`, the `check`
#                          recipe's own dependency list
#   - linecop-caps.json <- .linecop.yaml, via yq
#
# `gates.json` and `linecop-caps.json` get no consumer in this script's sibling
# task: they are the input to the follow-up
# truth/tasks/generate-the-root-markdown-from-typst-with-typlite.typ, which
# turns the root Markdown into Typst that can read them. Nothing here invents
# a consumer for either.
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

# --- crate-map.json ---------------------------------------------------------
# One `cargo metadata` call carries the whole dependency graph, so direct-vs-
# transitive for the substrate crates is a graph query, never a hardcoded
# claim (typst-world is transitive today, via typst-harvest — that has to stay
# derived, not asserted, or it silently goes stale the day something else
# pulls it in directly).
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

jq --arg substrate_re "$SUBSTRATE_REPO_RE" \
   --argjson archdeps "$ARCH_DEPS" \
   --arg channel "$CHANNEL" \
   --arg ws_version "$WS_VERSION" \
   --arg ws_edition "$WS_EDITION" \
   --arg ws_rust_version "$WS_RUST_VERSION" \
   --arg typst_lib_version "$TYPST_LIB_VERSION" \
'
. as $meta
| ($meta.workspace_members) as $ws
| ($meta.packages | INDEX(.name)) as $byname
# Every direct dep of a workspace member, by pkgid — a substrate crate absent
# from this set but present in $byname is reached only transitively.
| ([$meta.resolve.nodes[]
    | select(.id as $id | $ws | index($id) != null)
    | .deps[].pkg] | unique) as $direct_pkgs
| {
    workspace: [$meta.packages[]
      | select(.id as $id | $ws | index($id) != null)
      | {name, version, description}],
    substrate: [$meta.packages[]
      | . as $p
      | select(($ws | index($p.id)) == null)
      | select(($p.repository // "") | test($substrate_re))
      | {
          name: $p.name,
          version: $p.version,
          direct: (($direct_pkgs | index($p.id)) != null),
        }] | sort_by(.name),
    "arch-deps": [$archdeps[] as $n | {name: $n, version: ($byname[$n].version // null)}],
    "rust-toolchain": {channel: $channel},
    "workspace-package": {version: $ws_version, edition: $ws_edition, "rust-version": $ws_rust_version},
    "typst-lib": {version: $typst_lib_version},
  }
' <<<"$META" >"$OUT/crate-map.json"

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
  >"$OUT/dsl-fields.json"
rm -rf "$tmp"

# --- gates.json --------------------------------------------------------------
# `just check`'s own dependency list, unwrapped: most sub-gates run through
# `just mmz <name>` (memoization), reflected as `{"recipe":"mmz","arguments":
# ["<name>"]}`; `check-crap` is a direct dependency with no wrapper. Preserving
# `check`'s declared order (never re-sorted) so the generated table reads the
# same left-to-right as the Justfile.
JUST_DUMP="$(just --dump --dump-format json)"
jq '
  [.recipes.check.dependencies[]
    | if .recipe == "mmz" then .arguments[0] else .recipe end] as $names
  | {gates: [$names[] as $n | {name: $n, doc: .recipes[$n].doc}]}
' <<<"$JUST_DUMP" >"$OUT/gates.json"

# --- linecop-caps.json --------------------------------------------------------
# The exact `limits`/`overrides` .linecop.yaml carries — the source CONTRIBUTING.md
# quotes caps from. yq is the same jq-over-YAML wrapper generate.sh uses for the
# release workflow, so this stays consistent with the rest of the docs tooling.
yq '{limits, overrides}' .linecop.yaml >"$OUT/linecop-caps.json"

echo "generated $(find "$OUT" -maxdepth 1 -name '*.json' | wc -l) fact files under $OUT"
