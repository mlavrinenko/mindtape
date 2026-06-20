#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Resolve @local/mindtape correctly in a foreign worktree",
  status: done(2026, 6, 21)[Name-gated the in-tree lib override and added the typst.local-packages config seam; regression tests cover both.],
  priority: ice(impact: 9, confidence: 0.9, ease: 8),
  relates-to: (link("converge-speconaut.typ"),),
  tags: ("substrate", "speconaut"),
)

== Summary

`mindtape ls` / `mindtape check` run in a foreign worktree (any repo whose
`<root>/lib` is some other Typst package) silently report zero tasks, because the
in-tree lib override hijacks the `mindtape` package name. ADR 005 promises the
shared-checkout property — a MindTape-only user lists and validates tasks inside
a speconaut worktree unchanged — and this is the one blocker to it. Two parts:
fix the name hijack, and let a worktree register additional `@local` package
overrides in config so foreign packages (e.g. `@local/speconaut` used by an
`x-speconaut` extras helper) resolve without a global install.

The defect: `loader::lib_dir` (`crates/mindtape/src/loader.rs`) registers
`<root>/lib` as the `mindtape` package whenever that directory holds any
`typst.toml`, with no manifest-name check. `HarvestWorld::resolve_package`
(`crates/typst-harvest/src/world.rs`) keys local overrides by package name alone.
In a speconaut checkout `<root>/lib` is the `speconaut` package, so a task that
imports `@local/mindtape` is evaluated against speconaut's entry point, harvests
a `<speconaut>` marker instead of `mindtape.task`, and the loader drops it.

`HarvestWorld` already resolves any `@local/<name>:<version>` from the standard
local-package store (`$DATA_LOCAL/typst/packages/local/...`), so an installed
foreign package resolves today; what is missing is the name guard and a config
seam to point a package at an in-tree path instead of an install.

== Scope

- Gate the in-tree override on the manifest: only register `<root>/lib` for the
  `mindtape` package when `<root>/lib/typst.toml` actually declares
  `name = "mindtape"`. Otherwise register no `mindtape` override and let
  `resolve_package` fall through to the local-package store, which already works.
- Add a config seam for extra local-package overrides, e.g.
  `[[typst.local-packages]] name = "speconaut"  path = "lib"` in
  `.mindtape/config.toml`. The loader maps each entry through
  `HarvestWorld::with_local_package(name, root.join(path))`, so a worktree
  resolves `@local/<name>` from its tree without a global install. This is the
  "support @local libraries / whitelist them manually" requirement — generic, no
  consumer named in MindTape code.
- An `x-<ns>` slot may be any expression that evaluates to a value (plain dict or
  a helper call); MindTape harvests it opaquely. No payload-shape restriction.
- Confirm the dogfood path is unchanged: in MindTape's own repo `<root>/lib` is
  the `mindtape` package, so the override still fires and unreleased `0.2.0`
  resolves from the tree.
- Regression tests: (a) a synthetic root whose `lib/` is a non-`mindtape`
  package, containing a task that imports `@local/mindtape`, resolves the package
  from the store/override-free path rather than yielding zero tasks; (b) a
  configured `[[typst.local-packages]]` entry makes `@local/<name>` resolve from
  the named in-tree path. Use the existing harvest/world test scaffolding.
- Document the fallback: with no override and no config entry, a foreign tool
  needs the `mindtape` package (and any helper package) installed in its local
  store (e.g. via `mindtape init`). Note it where `init` is documented; do not
  special-case speconaut.

== Notes

The manifest-name guard is the minimal correctness fix; the config override seam
is the general "whitelist `@local` packages" capability. Pair with
`consumer-blind-root-markers.typ`, which scrubs the other `.mindtape/`-named
assumption from the substrate.
