#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Extract typst-world to its own repo (~/projects/home/typst-world) and publish",
  status: wip(2026, 7, 5)[
    Convergence merged (fe9d55a) — the crate now lives at crates/typst-world with
    default = [] (eval-only) and a render feature. Extracting. Name free (404).
  ],
  priority: ice(impact: 9, confidence: 0.7, ease: 5),
  tags: ("substrate", "tooling"),
  depends-on: (link("converge-typst-harvest-typst-world.typ"),),
)

== Summary

Move the converged `typst-world` (World + file-provider seam + Typst-free value
tree, optional render) into its own repo `~/projects/home/typst-world`
(cratemplate), then publish. This is the crate mindtape, jmav, and speconaut all
depend on for the Typst World, so it publishes before typst-harvest and before
the consumers repoint. Canonical home moves out of the jmav workspace.

== Scope

Crate facts (post-convergence): `World`/`WorldBuilder`, `FileProvider`/
`DiskProvider`/`BytesProvider`, `find_project_root`, `HVal`/`convert`/`format_date`;
`render`/`render_svg` behind a `render` feature. External deps: typst 0.15 family
(`typst`, `typst-eval`, `typst-library`; `typst-layout`/`typst-render`/`typst-svg`/
`typst-assets` only under `render`), `comemo`, `chrono`, `dirs`, `thiserror`.
Default features MUST stay eval-only (no fonts/render/network) so mindtape pays
nothing for render.

== Extraction procedure

1. `cargo generate gh:mlavrinenko/cratemplate` -> name `typst-world`, mlavrinenko,
   MIT. Lands `~/projects/home/typst-world`.
2. LIB — drop `[[bin]]`/`src/main.rs`; drop clap/anyhow/log/env_logger.
3. Copy the converged src/ (world.rs, provider.rs, value.rs, render.rs behind
   feature, lib.rs + `_tests.rs`).
4. Port metadata + the `[features]` (`render`, default eval-only) + deps. Reset
   version to 0.1.0. Set keywords `["typst","world","eval","provider","render"]`.
5. Reconcile lints (template `cast_*` deny).
6. `git init` + commit; GitHub repo `mlavrinenko/typst-world`; push.
7. Green gates with AND without `--features render`; `just nix-build`.
8. `cargo publish --dry-run` then `cargo publish`. Record version.

== Tests

- Gates green for default (eval-only) and `--features render`.
- `cargo tree --no-default-features` shows no typst-assets/render/network deps.

== Notes

- Coordinate with jmav: after publish, jmav repoints its workspace to the
  published `typst-world` and deletes its in-tree copy (separate jmav session).
- Orchestrator owns repointing mindtape.
