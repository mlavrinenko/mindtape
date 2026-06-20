#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Drop the .mindtape root marker from typst-harvest",
  status: done(2026, 6, 21)[Dropped the .mindtape marker from typst-harvest find_project_root; tests now mark roots generically.],
  priority: ice(impact: 6, confidence: 0.9, ease: 9),
  relates-to: (link("converge-speconaut.typ"),),
  tags: ("substrate",),
)

== Summary

`typst-harvest::find_project_root` (`crates/typst-harvest/src/world.rs`)
hardcodes `.mindtape/` as its first root-detection marker. `typst-harvest` is a
consumer-blind substrate crate — destined for its own repo, already path-depended
by speconaut — and must not name a consumer. Scrub the `.mindtape/` assumption so
the crate stays domain-free before a second tool hardens the dependency.

== Scope

- Remove `.mindtape/` from `find_project_root`'s marker list; keep the generic
  markers (`Cargo.toml`, `lib/`, `.git`) and the `start` fallback.
- The `.mindtape/`-aware walk already lives in the domain layer
  (`loader::find_root`, `crates/mindtape/src/loader.rs`), which is where the
  MindTape-specific knowledge belongs. Confirm the CLI's root discovery routes
  through the domain `find_root`, not the substrate helper, for the project-root
  case; reserve `find_project_root` for the generic VCS/manifest case.
- Update the doc comment to drop the `.mindtape/` mention.
- Adjust `world_tests.rs` if any case asserts the `.mindtape/` marker.

== Notes

Hygiene companion to `foreign-worktree-package-resolution.typ`; together they
remove the two places where a "consumer-blind" crate still assumes the MindTape
consumer. Not strictly blocking convergence, but cheap to do now and awkward to
change once speconaut pins the crate.
