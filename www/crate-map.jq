# www/crate-map.jq — the jq program www/generate-facts.sh runs over `cargo
# metadata --format-version 1` (as $META, piped in) to build crate-map.json.
# The third sibling of www/gates.jq and www/sweep-flags.jq, and split out for
# the same reason those two were: www/generate-facts.sh has to stay under its
# own linecop cap, and this was the last derivation still written inline
# there — the one that had to move for site-pages.json to fit at all.
#
# Everything it needs that is NOT in the metadata comes in as an argument
# ($substrate_re, $archdeps, $channel, $ws_version, $ws_edition,
# $ws_rust_version, $typst_lib_version); those are read from the manifests and
# rust-toolchain.toml by the shell, and argued at their assignments there.
#
# One `cargo metadata` call carries the whole dependency graph, so direct-vs-
# transitive for the substrate crates is a graph query, never a hardcoded
# claim (typst-world is transitive today, via typst-harvest — that has to stay
# derived, not asserted, or it silently goes stale the day something else
# pulls it in directly).
#
# Living in its own file also lifts a constraint this program used to carry:
# it was a single-quoted shell string, so it could not contain an apostrophe.
# Prose here is free to.

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
