#import "@local/mindtape:0.3.0": *

#show: task.with(
  title: "adopt typst-cleanup pluggable scan",
  priority: ice(
    impact: 5,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("substrate", "cli"),
  status: done(
    2026,
    7,
    12,
  )[rewired onto typst\_cleanup::plan via Policy::scan. Local skeleton deleted],
)

== Summary

mt rm reimplements typst-cleanup's plan skeleton locally (mint + survivor scan + atomic assembly) because the crate's scan was link()-only and could not see the links: helper edges. Once typst-cleanup 0.2 makes the scan pluggable via Policy::scan, rewire mt rm to hand TaskLinks back to typst_cleanup::plan and delete the local copy.

== Scope

Blocked on the typst-cleanup pluggable-scan release
(github.com/mlavrinenko/typst-cleanup#1: make the survivor scan pluggable via
`Policy`). Do this once that release reaches crates.io.

- Bump the `typst-cleanup` dependency to the pluggable-scan release (0.2.x) in
  `crates/mindtape-cli/Cargo.toml`.
- Implement `Policy::scan` on `TaskLinks` via `mindtape::link_target_spans`
  (map `TargetSpan` → `typst_cleanup::Target`). Adopt the renamed
  `identity` (was `link_identity`). `scan` + `identity` + `tombstone` make
  `TaskLinks` a real policy again, handed back to the crate.
- Delete the local copy in `crates/mindtape-cli/src/rm.rs`
  (`plan`, `mint_tombstone`, `plan_rewrite`). Call
  `typst_cleanup::plan(&retire, &survivors, git_root, &TaskLinks)`, and let the
  crate re-own mint, dirty-tree guard, atomic assembly, and `apply`.
- Keep `rm_tests` and `rm_e2e` green: every inbound edge kind (depends-on,
  related, custom `edge`) still points at the SWHID tombstone and `mt check`
  stays green.
- `mindtape::link_target_spans` stays in `mindtape` (it knows the mindtape DSL
  helper names), and the crate stays DSL-agnostic. A candidate for speconaut to
  converge onto, with the same shape (cleanup over `git-ref(...)`/`scenario(...)`).
