#import "@local/mindtape:0.3.0": *

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
  status: done(
    2026,
    8,
    8,
  )[18 `arg` modules added as `cli-docs` sources, plus 9 previously untracked dependents. Bite-tested against a scratch lock, and it caught the next real change immediately],
)

== Summary

The outdatty `cli-docs` group names `crates/mindtape-cli/src/cli.rs` as its CLI
source, but every command's argument struct lives in its own module: `AddArgs`
is in `add.rs`, `LsArgs` in `ls.rs`, and so on. `cli.rs` only names the types. So
adding two user-facing flags to `mt add` left every declared source untouched,
and no group went stale. The drift gate reported all six groups current while the docs were, for
a moment, actually behind. We found it by checking why `cli-docs` was
unexpectedly green, not from any gate.

== Scope

- Add the per-command argument modules to the `cli-docs` group's sources, so a
  flag change forces the same dependent review a command change already does.
- The new `check-doc-coverage` gate closes the missing-command hole. This is the
  missing-flag hole beside it, and neither covers the other.
- Weigh listing the modules explicitly against a glob. An explicit list rots
  when someone adds a command, and a glob over-triggers on internal churn.
  Whichever wins, say why in the group comment. That file explains its own
  reasoning throughout, and this decision should not need re-deriving.
- The same audit applies on the dependent side. `www/content/templates.typ`
  documents `mt add` slot behaviour and appears in no group at all, which is how
  it kept saying text slots were the only escaping story after the `typst` kind
  landed. A person reading the agent's report caught that, not the gate. Sweep
  every page under `www/content/` and every file under `skills/` for the same
  omission, and add what is missing.
