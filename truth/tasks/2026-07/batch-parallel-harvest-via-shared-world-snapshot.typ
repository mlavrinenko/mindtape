#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "batch parallel harvest via shared world snapshot",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 4,
  ),
  tags: ("mt", "substrate"),
  status: done(
    2026,
    7,
    16,
  )[landed: typst-world 0.2.0 + typst-harvest 0.2.0 on crates.io, loader adopts `batch_harvest` (~13-19x at 2000 tasks)],
)

== Summary

The loader harvests each file sequentially with a fresh HarvestWorld, a fresh Typst Library, and a prelude re-eval per file (~30ms x N). Replace that with a shared-library + source-snapshot + parallel batch harvest. Port the proven typst-batch design into the substrate we own. typst-world grows a shared standard Library and an immutable Arc<SourceSnapshot>, typst-harvest grows a rayon `batch_harvest` behind an optional feature, and the mindtape loader calls it. The cold path drops from ~1s to ~150-300ms on this backlog. The approach supersedes the rejected rayon-in-mindtape one. The capability then lands in the reusable crates, not the consumer.

== Scope

- typst-world: share one standard `Library` across worlds (a shareable `LazyHash<Library>`, cf. typst-batch `GLOBAL_LIBRARY`) instead of `Library::default()` per world. That kills the ~7.7ms/file fixed cost and lets `comemo` reuse.
- typst-world: immutable `SourceSnapshot` (pre-read + parse each source once, Arc-shared, lock-free), plus a world constructor over a shared snapshot + shared library. It keeps the existing `@local/<name>` multi-package overrides and `HVal` conversion. The 458-line prelude then parses and eval-caches once, and every file reuses it.
- typst-harvest: `batch_harvest` over many files with rayon `par_iter`, gated behind an optional `batch` feature (keep rayon optional). Fonts stay off (eval-only).
- mindtape loader: swap the sequential loop for the batch API. Drop the fresh-world-per-file path.
- No new crate: the seam is clean. World construction (shared lib + snapshot) belongs in typst-world, batch orchestration in typst-harvest. Reject a third crate unless real shared code appears.
- Do not depend on tola-rs typst-batch directly. It pulls typst-html/typst-kit/typst-svg, uses its own value model, and has single-dir `@local` resolution. That is a wrong fit for eval-only harvest with named local packages. Port the design, own the code.
- Coordinated release: land typst-world then typst-harvest (crates.io), then bump the mindtape dependencies. User releases the crates.
- Tests: batch harvest matches sequential harvest task-for-task. Snapshot handles a shared import. A per-file eval error isolates to its file.
