#!/usr/bin/env bash
# Use this flake the way an adopter does: lock examples/nix-adopter against the
# working tree, cap the nodes Mindtape adds to that lock, then run `mt` from the
# fixture's dev shell on a `$HOME` that never ran `mt install`, and `nix run` this
# flake.
set -euo pipefail

# Mindtape's own node, nixpkgs and what `packages.default` builds from. A dev
# input that leaks into the outputs adopters read pushes the lock past it.
MAX_NODES=6

root=$(git rev-parse --show-toplevel)
# A git flake sees only files git knows, so new ones are marked intent-to-add.
git -C "$root" ls-files -z --others --exclude-standard | xargs -0 -r git -C "$root" add --intent-to-add --

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir "$work/shell" "$work/project" "$work/home"
cp "$root/examples/nix-adopter/flake.nix" "$work/shell/"

# Quiet on success: the lock prints every input it adds.
if ! nix flake lock "$work/shell" --allow-dirty-locks --override-input mindtape "git+file://$root" 2>"$work/lock.log"; then
  cat "$work/lock.log" >&2
  exit 1
fi
nodes=$(jq '.nodes | length - 1' "$work/shell/flake.lock")
if [ "$nodes" -gt "$MAX_NODES" ]; then
  inputs=$(jq -r '.nodes.mindtape.inputs | keys | join(", ")' "$work/shell/flake.lock")
  echo "smoke-flake: an adopter's lock gains $nodes nodes, over the cap of $MAX_NODES; mindtape's inputs: $inputs" >&2
  exit 1
fi

cd "$work/project"
git init -q
# The inner script expands inside the dev shell, where `mt` is the flake's.
# shellcheck disable=SC2016
nix develop "$work/shell" --command env -u XDG_DATA_HOME -u TYPST_PACKAGE_PATH -u MINDTAPE_DATA_HOME \
  HOME="$work/home" bash -euo pipefail -c '
    case $(command -v mt) in
      /nix/store/*) ;;
      *) echo "smoke-flake: mt resolves to $(command -v mt), not the flake'\''s" >&2; exit 1 ;;
    esac
    mt init --no-input >/dev/null
    mt add "Try the flake" >/dev/null
    out=$(mt check 2>&1)
    if [ "$out" != "ok: 1 task" ]; then
      printf "smoke-flake: mt check printed:\n%s\n" "$out" >&2
      exit 1
    fi
  '

nix run "git+file://$root" -- --version >/dev/null
echo "ok: an adopter locks $nodes nodes and runs mt"
