#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "stop duplicating mt add's slug in mindtape's slug_from_title",
  priority: ice(
    impact: 3,
    confidence: 0.9,
    ease: 9,
  ),
  difficulty: 1,
  tags: ("cli",),
  estimate: (files: 2),
  assignee: "agent",
  status: done(
    2026,
    9,
    10,
  )[Fixed at 2094612, folded into the apostrophe fix since both landed in the same commit: `add_values::slug` now delegates to `mindtape::slug_from_title` instead of re-implementing it. `just check-dry`'s jscpd clone is gone (0 clones, was 1 at 72 tokens).],
)

== Summary

`crates/mindtape-cli/src/add_values.rs`'s `slug` and `crates/mindtape/src/import_map/stem.rs`'s `slug_from_title` are byte-for-byte identical function bodies, kept as two hand-synced copies because the CLI crate depends on `mindtape`, not the reverse — so the story went, `mt add` could not reach `mt import`'s copy. That premise was already false: `mindtape-cli` depends on `mindtape`, and `mindtape` already re-exports `slug_from_title` publicly. Fixing the apostrophe-slugging bug in both copies identically (see [drop-an-apostrophe-inside-a-word-instead-of-slugging-it-to-a-dash](drop-an-apostrophe-inside-a-word-instead-of-slugging-it-to-a-dash.typ)) grew the pair past `just check-dry`'s jscpd clone floor, which exists precisely to force a shared helper instead of a second hand-synced copy.

== Scope

Make `add_values::slug` call `mindtape::slug_from_title` instead of re-implementing it, deleting the duplicate loop. Keep `slug` as a thin wrapper (same name, same signature) so its call sites and its own regression test are undisturbed. Update `stem.rs`'s module doc, which currently claims the duplication is deliberate, to drop that claim. Re-run `just check-dry` to confirm the clone is gone.
