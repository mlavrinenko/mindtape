#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "batch parallel harvest via shared world snapshot",
  priority: ice(
    impact: 8,
    confidence: 0.8,
    ease: 4,
  ),
  tags: ("mt", "substrate",),
  status: wip(2026, 7, 16)[benchmark harness landed (examples/large, just bench); implementing batch\_harvest next],
)

== Summary

Replace the loader's sequential per-file harvest (a fresh HarvestWorld + fresh Typst Library + prelude re-eval per file, ~30ms x N) with a shared-library + source-snapshot + parallel batch harvest, porting the proven typst-batch design into the substrate we own: typst-world grows a shared standard Library and an immutable Arc<SourceSnapshot>, typst-harvest grows a rayon batch_harvest behind an optional feature, and the mindtape loader calls it. Cold-path win (~1s to ~150-300ms on this backlog); supersedes the rejected rayon-in-mindtape approach so the capability lands in the reusable crates, not the consumer.

== Scope

- typst-world: share one standard `Library` across worlds (a shareable `LazyHash<Library>`, cf. typst-batch `GLOBAL_LIBRARY`) instead of `Library::default()` per world — kills the ~7.7ms/file fixed cost and lets comemo reuse.
- typst-world: immutable `SourceSnapshot` (pre-read + parse each source once, Arc-shared, lock-free), plus a world constructor over a shared snapshot + shared library that keeps the existing `@local/<name>` multi-package overrides and HVal conversion. So the 458-line prelude is parsed/eval-cached once, reused across files.
- typst-harvest: `batch_harvest` over many files with rayon `par_iter`, gated behind an optional `batch` feature (keep rayon optional). Fonts stay off (eval-only).
- mindtape loader: swap the sequential loop for the batch API; drop the fresh-world-per-file path.
- No new crate: the seam is clean — World construction (shared lib + snapshot) belongs in typst-world, batch orchestration in typst-harvest. Reject a third crate unless real shared code appears.
- Do NOT depend on tola-rs typst-batch directly: it pulls typst-html/typst-kit/typst-svg, uses its own value model, and has single-dir `@local` resolution — wrong fit for eval-only harvest with named local packages. Port the design, own the code.
- Coordinated release: land typst-world then typst-harvest (crates.io), then bump mindtape deps. User releases the crates.
- Tests: batch harvest matches sequential harvest task-for-task; snapshot handles a shared import; per-file eval error isolates to its file.
