#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "split config rs before it blocks the next change",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 5,
  ),
  difficulty: 4,
  tags: ("cli", "tooling"),
  estimate: (files: 4),
  status: done(
    2026,
    8,
    8,
  )[extracted the rm/retire facet into config_rm.rs; 499 -> 463, no prose trimmed],
)

== Summary

`crates/mindtape/src/config.rs` sits at 499 of its 500-line cap — one line of headroom. The `split-filter-rs-before-it-blocks-the-next-change` task named this file explicitly as owed the same treatment, and the next config-carrying feature (the `mt add` dedup suffix option) lands here. A cap with one line left does not push back on scope growth; it pushes back on the next correct change.

== Scope

- Find a real seam and extract it into a sibling module, in the shape the existing `config_ls.rs` / `config_views.rs` / `config_required.rs` split already established.
- Re-export from `config.rs` so no call site outside the crate changes.
- Split the test module alongside if it is near its own cap.
- Not in scope: `rm.rs` (494) and `rm_gitless.rs` (485), which are in the same position and get their own tasks — do not bundle them.
