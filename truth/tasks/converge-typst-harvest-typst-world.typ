#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Converge typst-harvest and jmav typst-world into one shared World crate",
  status: proposed(2026, 7, 5)[
    The fork was the accident: typst-world was meant to be a dependency of
    typst-harvest (extracted World layer), not a copy of it. Restore that.
  ],
  priority: ice(impact: 9, confidence: 0.6, ease: 3),
  tags: ("substrate", "tooling"),
  depends-on: (link("typst-015-bump.typ"),),
)

== Summary

Today the same eval-only Typst World lives twice: `crates/typst-harvest`
(0.14 here) and `~/projects/home/jmav/crates/typst-world` (0.15, a superset fork
— identical `value.rs`, `harvest.rs` differing only by the 0.15 API). Collapse
them into ONE World crate, `typst-world`, that both mindtape and jmav (and
speconaut) depend on. Restore the intended layering: typst-world owns the
World + file-provider seam; the harvest logic sits on top.

== Scope

Target shape:

- `typst-world` (the shared crate, jmav's is the base — it has the `FileProvider`
  seam, `BytesProvider` for wasm, render, `sys.inputs`, on 0.15). It owns:
  `World`/`WorldBuilder`, `FileProvider`/`DiskProvider`/`BytesProvider`,
  `find_project_root`, and the `HVal`/`convert`/`format_date` value tree (shared,
  Typst-free — keep OUR date handling; do NOT switch to serde_json, per the
  typst-batch spike finding that datetimes serialize to unusable repr strings).
- `typst-harvest` becomes THIN: `harvest()` + `Harvest`/`Marker`/`Location`,
  depending on `typst-world` for the World and value tree. Delete its `world.rs`
  and `value.rs` (moved to typst-world). Keep the `with_marker`/`with_label`
  filters and marker model.
- Decide and document: does `harvest()` live in typst-world (jmav bundles it) or
  in typst-harvest (mindtape's split)? Recommendation: keep `harvest()` in
  typst-harvest so typst-world stays a pure World+value substrate; jmav then
  depends on typst-harvest for harvesting, or re-exports. Confirm with jmav's
  needs before finalising.

Reconcile the two implementations line by line (they are 95% identical); the
divergences are the 0.15 API (already handled by task typst-015-bump) and jmav's
provider seam + render. mindtape does not need render — gate render behind a
feature (`render`) so the eval-only consumer pays nothing.

Preserve mindtape's load-bearing surface (verified consumers in
`crates/mindtape/src/loader.rs`, `crates/mindtape-cli/src/eval.rs`):
`World::with_root` / `for_file`, `with_local_package(name, dir)` (the @local dev
override), `harvest()`, `find_project_root`. mindtape does NOT use marker
location — do not over-preserve it.

== Tests

- `just check` green; `just nix-build` green.
- `mt ls` / `mt check` output identical to before convergence.
- typst-world builds with `--no-default-features` (eval-only, no fonts/render, no
  typst-assets/render deps) AND with `--features render`.
- A jmav-side smoke check that its renderer still builds against the shared crate
  (coordinate; jmav is a separate repo/session).

== Notes

- This lands in-tree (mindtape workspace) first; extraction to its own repo is the
  separate `extract-typst-world.typ` task.
- Keep file boundaries disjoint from the extraction wave; this task only reshapes
  crates, does not move them.
