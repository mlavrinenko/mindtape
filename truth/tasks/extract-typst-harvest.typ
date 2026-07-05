#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Extract typst-harvest to its own repo (~/projects/home/typst-harvest) and publish",
  status: wip(2026, 7, 5)[
    typst-world 0.1.0 is live on crates.io — dry-run against it is clean now.
    Extracting via the fixed cratemplate (crate_kind=lib). Agent dispatched.
  ],
  priority: ice(impact: 7, confidence: 0.7, ease: 6),
  tags: ("substrate", "tooling"),
  depends-on: (
    link("converge-typst-harvest-typst-world.typ"),
    link("extract-typst-world.typ"),
  ),
)

== Summary

Move the thinned `typst-harvest` (metadata-marker harvesting on top of
`typst-world`) into `~/projects/home/typst-harvest` (cratemplate), then publish.
Depends on `typst-world` being published first (crates.io requires published
deps), hence the `extract-typst-world` dependency.

== Scope

Crate facts (post-convergence): `harvest()` + `Harvest`/`Marker`/`Location` +
`with_marker`/`with_label`; World and value tree come from `typst-world`. External
deps: `typst-world = "<published>"`, `typst`/`typst-eval`/`typst-library` (0.15) as
needed for eval + `MetadataElem` traversal, `thiserror`. keywords per current
manifest.

== Extraction procedure

1. `cargo generate gh:mlavrinenko/cratemplate` -> name `typst-harvest`,
   mlavrinenko, MIT. Lands `~/projects/home/typst-harvest`.
2. LIB — drop `[[bin]]`/`src/main.rs`; drop clap/anyhow/log/env_logger.
3. Copy the converged thin src/ (harvest.rs, lib.rs, `_tests.rs`). world.rs/value.rs
   are gone (now in typst-world).
4. Port metadata + deps; point `typst-world` at its published version. Reset
   version to 0.1.0.
5. Reconcile lints (template `cast_*` deny).
6. `git init` + commit; GitHub repo `mlavrinenko/typst-harvest`; push.
7. `just check` + `just nix-build` green.
8. `cargo publish --dry-run` then `cargo publish`. Record version.

== Tests

- Gates green; the `harvest` doctest in lib.rs still passes against the published
  typst-world; `cargo publish --dry-run` clean.

== Notes

- Orchestrator owns repointing mindtape.
