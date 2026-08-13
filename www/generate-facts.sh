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
# gap between them. `just docs-gen` then calls this again — idempotent, and it
# keeps the recipe honest about what it produces — and `just check-doc-facts`
# and `docs/generate-md.sh` call it directly, so each is self-contained and
# never depends on a prior docs-gen run.
#
# Five artifacts, five sources:
#   - crate-map.json    <- `cargo metadata`, rust-toolchain.toml, the
#                          [workspace.package] table, lib/typst.toml
#   - dsl-fields.json   <- lib/prelude.typ's `task` signature, read back
#                          through its own `<mindtape>` marker
#   - gates.json        <- `just --dump --dump-format json` for the
#                          `[group("gate")]` tags, `just --summary --unsorted`
#                          for source order
#   - just.typ          <- the same `just --dump` as gates.json (one
#                          shell-out, two artifacts): a Typst module, one
#                          function per public recipe
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

# Same lock as www/generate.sh, same reason: `just check` is [parallel] and
# check-doc-facts, docs-md-check and docs-check can all be inside this
# directory at once. Re-exec under flock unless a caller already holds it —
# www/generate.sh exports the variable before calling this script, so the
# wipe-then-restore there stays ONE critical section instead of two.
if [ -z "$MINDTAPE_GENERATED_LOCK" ]; then
  export MINDTAPE_GENERATED_LOCK=1
  exec flock "$REPO/target/www-generated.lock" "$0" "$@"
fi

# Every artifact below is written through a temp file and moved into place, so
# a reader holding no lock (a Typst source mid-render) sees the previous
# complete file or the next one, never a truncated or absent one. `mv` within
# one filesystem is atomic; a redirect straight onto the destination is not,
# and would also leave a zero-byte file behind when the producing jq program
# calls halt_error (the gate-group cross-check below does).
#
# The parse check is what makes that guarantee hold under a FAILING producer,
# not just a slow one: a jq that dies mid-stream still closes the pipe, so the
# temp file exists and is partial. Installing a partial fact file is worse
# than installing none — the reader gets a plausible-looking JSON prefix
# instead of an error. Every artifact here is JSON, so `jq -e .` is the test.
emit() {
  local name="$1" tmp="$OUT/.$1.tmp.$$"
  cat >"$tmp"
  if ! jq -e . "$tmp" >/dev/null 2>&1; then
    rm -f "$tmp"
    echo "generate-facts.sh: refusing to install malformed $name" >&2
    return 1
  fi
  mv -f "$tmp" "$OUT/$name"
}

# Same temp-file-plus-atomic-mv discipline as emit() above, for an artifact
# that is not JSON — `jq -e .` has nothing to parse. just.typ's own producer
# below builds the whole file as ONE jq string value before printing
# anything, so a halted run (halt_error) writes nothing rather than a
# truncated prefix; refusing empty content is the raw equivalent of emit()'s
# parse check.
emit_raw() {
  local name="$1" tmp="$OUT/.$1.tmp.$$"
  cat >"$tmp"
  if [ ! -s "$tmp" ]; then
    rm -f "$tmp"
    echo "generate-facts.sh: refusing to install empty $name" >&2
    return 1
  fi
  mv -f "$tmp" "$OUT/$name"
}

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
| ($meta.packages | map({key: .id, value: .name}) | from_entries) as $idname
# pkgid -> the names of the packages that depend on it. A transitively reached
# substrate crate is documented as "via <parent>", and WHICH parent pulls it is
# a fact with a source (the resolve graph), so it is read rather than asserted.
# AGENTS.md carried "typst-world is pulled transitively via typst-harvest" by
# hand, which was true and had no way to STAY true.
# (No apostrophes in this jq program: it is a single-quoted shell string.)
| (reduce ($meta.resolve.nodes[] | . as $n | $n.deps[] | {parent: $idname[$n.id], child: .pkg})
    as $e ({}; .[$e.child] += [$e.parent])) as $parents_of
| {
    workspace: [$meta.packages[]
      | select(.id as $id | $ws | index($id) != null)
      | {name, version, description}],
    substrate: [$meta.packages[]
      | . as $p
      | select(($ws | index($p.id)) == null)
      | select(($p.repository // "") | test($substrate_re))
      | (($direct_pkgs | index($p.id)) != null) as $is_direct
      | {
          name: $p.name,
          version: $p.version,
          direct: $is_direct,
          # Only meaningful when not direct; null keeps the shape uniform.
          via: (if $is_direct then null
                else (($parents_of[$p.id] // []) | unique | sort) end),
        }] | sort_by(.name),
    "arch-deps": [$archdeps[] as $n | {name: $n, version: ($byname[$n].version // null)}],
    "rust-toolchain": {channel: $channel},
    "workspace-package": {version: $ws_version, edition: $ws_edition, "rust-version": $ws_rust_version},
    "typst-lib": {version: $typst_lib_version},
  }
' <<<"$META" | emit crate-map.json

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

# --- gates.json --------------------------------------------------------------
# Gate list + row order from the Justfile's `[group("gate")]` tags and its
# own recipe order, cross-checked against `check`'s dependency list; each
# gate also carries a derived `command` (the recipe's own body, when short
# enough to quote). The jq program is long enough to blow this script's own
# linecop cap, so it lives in www/gates.jq (argued in place there) instead
# of inline.
JUST_DUMP="$(just --dump --dump-format json)"
SUMMARY_ORDER="$(just --summary --unsorted | jq -R 'split(" ") | map(select(length > 0))')"
jq --argjson order "$SUMMARY_ORDER" -f "$SCRIPT_DIR/gates.jq" <<<"$JUST_DUMP" | emit gates.json

# --- just.typ ------------------------------------------------------------
# Reuses $JUST_DUMP above — one `just --dump` shell-out, two artifacts. The
# module and its functions are explained in JUST_TYP_HEADER below, which
# ships verbatim inside the artifact itself so that explanation travels with
# the file. What is NOT in the shipped header: the identifier collision
# check — two recipes mangling to the same `-`-joined identifier fail the
# jq program below with halt_error, naming both, rather than one silently
# shadowing the other.
JUST_TYP_HEADER="$(cat <<'HDR'
// www/generated/just.typ — generated by www/generate-facts.sh from `just
// --dump --dump-format json`; do not edit, edit the Justfile.
//
// A Typst MODULE, not a dict — Typst reads a dict field call as a method
// call and errors "type dictionary has no method" (verified). One function
// per PUBLIC recipe (private ones, `_name`, are excluded from the docs
// surface, same as check-doc-coverage's `hide = true` exclusion). Each
// renders the invocation as a raw span: `#just.check()` -> `just check`;
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
  [.recipes[] | select(.private == false)
    | {name, id: (.namepath | gsub("::"; "-"))}] as $recipes
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

# --- linecop-caps.json --------------------------------------------------------
# The exact `limits`/`overrides` .linecop.yaml carries — the source CONTRIBUTING.md
# quotes caps from. yq is the same jq-over-YAML wrapper generate.sh uses for the
# release workflow, so this stays consistent with the rest of the docs tooling.
yq '{limits, overrides}' .linecop.yaml | emit linecop-caps.json

echo "generated $(find "$OUT" -maxdepth 1 \( -name '*.json' -o -name '*.typ' \) | wc -l) fact files under $OUT"
