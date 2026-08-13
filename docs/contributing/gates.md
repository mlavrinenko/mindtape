# Scoping a gate

Deep dive behind one line of CONTRIBUTING.md's Quality Gates section: the one
sanctioned way to commit without a fully fresh `just check`.

## What the pre-commit hook asserts

The hook installed by `nix develop` asserts the gate is fresh
(`mmz --is-fresh --tag gate`) and checks task files. A commit whose gate run is
stale is refused.

## The escape hatch

`MINDTAPE_GATE_SCOPE=<reason> git commit` skips only the freshness assertion.
Everything else still runs: `typstyle --check truth/tasks` and `mt check` are
unaffected.

It is not a silent skip. `.githooks/commit-msg` refuses the commit unless the
message carries a matching `Gate-Scope: <reason>` trailer, so a scoped commit
never reads like a fully gated one in `git log`. The record of what was not
verified travels with the commit that did not verify it.

## When it is legitimate

One case: a delegated worktree whose full aggregate runs at integration. The
gate is not being skipped, it is being deferred to a run that provably covers the
merged result — which is the only run that proves the integration anyway. A
branch that passes its gate in isolation can still break the integrated tree.

It is never legitimate to dodge a gate that is actually red. If the gate fails,
the commit is wrong, and a trailer saying so does not make it right.
