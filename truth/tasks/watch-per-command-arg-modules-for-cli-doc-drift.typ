#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "watch per command arg modules for cli doc drift",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 3,
  tags: ("gating", "docs"),
  estimate: (files: 3),
  status: proposed(2026, 8, 8),
)

== Summary

The outdatty `cli-docs` group names `crates/mindtape-cli/src/cli.rs` as its CLI source, but every command`s argument struct lives in its own module — `AddArgs` is in `add.rs`, `LsArgs` in `ls.rs`, and so on. `cli.rs` only names the types. So adding two user-facing flags to `mt add` changed no declared source and busted no group: the drift gate reported all six groups current while the docs were, for a moment, genuinely behind. Found by checking why `cli-docs` was unexpectedly green, not by any gate.

== Scope

- Add the per-command argument modules to the `cli-docs` group`s sources so a flag change forces the same dependent review a command change already does.
- The new `check-doc-coverage` gate closes the missing-command hole; this is the missing-flag hole beside it, and neither covers the other.
- Weigh listing the modules explicitly against a glob: an explicit list rots when a command is added, a glob over-triggers on internal churn. Whichever is chosen, say why in the group comment — that file explains its own reasoning throughout and this decision should not have to be re-derived.
