#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "Bump the mindtape workspace from typst 0.14 to 0.15",
  status: wip(2026, 7, 5)[
    Foundational: unblocks convergence and the substrate extraction. Supersedes
    the CLAUDE.md branch directive "don't bump typst" — the owner approved 0.15.
    Worktree agent dispatched on branch agent/typst-015-bump.
  ],
  priority: ice(impact: 9, confidence: 0.8, ease: 5),
  tags: ("substrate", "tooling"),
)

== Summary

Migrate every crate in this workspace from the typst 0.14 crate family to 0.15,
no behaviour change. This is a pure compiler-version migration; it does NOT touch
the Typst PACKAGE version (`lib/typst.toml` stays 0.2.0) or the DSL. Reference
implementation exists and compiles: `~/projects/home/jmav/crates/typst-world`
already runs this exact migration for the World + harvest code — read
`src/{world,harvest,provider,value}.rs` there as the crib.

== Scope

Bump these dependencies to `0.15` across the workspace manifests:
`typst`, `typst-eval`, `typst-library` (typst-harvest); `typst-syntax`
(typst-cst, typst-edit). Keep `comemo = "0.5"`, `chrono`, `dirs`, `thiserror`.

Crates touched: typst-harvest, typst-cst, typst-edit, and the consumers
(mindtape, mindtape-cli) that surface any changed type. typst-cleanup,
swhid-mint, artifact-gate carry no typst dep and should not change (verify).

Do NOT yet adopt jmav's `FileProvider` seam or dedupe the World — that is the
separate convergence task. Keep typst-harvest's current module shape; only move
its API calls to 0.15.

== Known 0.14 to 0.15 API deltas

From the verified jmav diff (`crates/typst-harvest` vs jmav `typst-world`):

- `typst_eval::eval(&ROUTINES, world.track(), ...)` drops `&ROUTINES` and gains
  the library: `typst_eval::eval(world.track(), world.library(), traced.track(),
  sink.track_mut(), route.track(), &source)`.
- `FileId::new(None, VirtualPath::new(rel))` becomes
  `FileId::new(RootedPath::new(VirtualRoot::Project, vpath))`, and
  `VirtualPath::new` now returns `Result` (handle the error).
- Package id: `FileId::new(Some(spec), vpath)` becomes
  `RootedPath::new(VirtualRoot::Package(spec), vpath)`.
- `World::today(&self, offset: Option<i64>)` becomes
  `Option<typst::foundations::Duration>`.
- `source.range(span)` becomes `world.range(span)` via `typst::WorldExt`.
- `id.vpath().as_rooted_path()` becomes `id.vpath().get_with_slash()`.
- `id.package()` (Option) becomes `id.root()` matched on
  `VirtualRoot::{Package(spec), Project}`.
- Path realize: `vpath.resolve(&base)` becomes `vpath.realize(&base)`.
- typst-syntax (cst/edit): expect `Source`, `LinkedNode`, `SyntaxKind`, `ast`
  to be largely stable; fix whatever the compiler flags.

== Tests

- `just check` green on 0.15 (fmt, clippy -D, tests, machete, linecop, jscpd,
  outdatty). Apply `cargo fmt` first.
- `just nix-build` green in the clean sandbox (typst 0.15 vendors cleanly).
- `mt ls` / `mt check` against `truth/tasks/` produce identical output to 0.14
  (spot-check a couple tasks; dates, priority bands, links unchanged).
- No new deps leak in (no typst-kit / network / openssl — verify `cargo tree`).

== Notes

- If `outdatty` flags drift, update dependents then `just outdatty-update`.
- Commit as `chore` or `build`; footer `Refs: truth/tasks/typst-015-bump.typ`.
