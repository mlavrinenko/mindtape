#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "site-pages-fixture-check fails on a cold checkout",
  priority: ice(
    impact: 5,
    confidence: 0.9,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("gating",),
  estimate: (files: 2),
  assignee: "agent",
  status: proposed(2026, 8, 21),
)

== Summary

The first `just check` in a freshly forked worktree fails on
`site-pages-fixture-check` alone, and passes on every run after. Observed
2026-08-21 by a delegated agent working in a `just fork` worktree, which
correctly re-ran the recipe directly and got a pass.

The gate replicates `www/` with `cp -r` and runs the copy's own
`generate-site-pages.sh`. That producer needs `www/generated/*.json`, which is
gitignored (`www/.gitignore`) and produced by the `docs-gen` arm — running in
PARALLEL with this one, because `just check` runs its arms `[parallel]`. The
script's own header states the assumption it breaks on:

#quote(block: true)[
  `www/generated/` rides along in the same `cp -r` — it is already on disk by
  the time any `just check` arm reaches this gate
]

That holds in a warm checkout, where an earlier run left the directory behind,
and only there. `just fork` warms no build cache by design, so a delegated
worktree hits this every time — and a fresh clone does too.

== Why it matters more than one re-run

The failure is indistinguishable from the agent's own breakage. It arrives on
the run that decides whether a delegated change is green, names a gate the
change never touched, and the correct response — re-run it — is also what a
flaky-test habit looks like. One agent this session triaged it right; the next
one has no reason to.

A gate whose result depends on whether an unrelated parallel arm has finished
is not pinning anything on the run where it matters.

== Scope

- Make `site-pages-fixture-check` carry its own inputs: depend on `docs-gen`
  the way `docs::check` does, or have the replica generate (or stub) the
  `www/generated/*.json` its producer reads
- Prefer whichever keeps the arm independent — a `[parallel]` aggregate must
  not encode an ordering between two arms that do not declare one
- Reproduce first: the bug only shows with `www/generated/` absent, so delete
  it (or fork a fresh worktree) before and after, and prove the gate fails then
  passes
- Keep `.mmz/config.yaml`'s scope for the recipe in step if its inputs change
- Out of scope: warming caches in `just fork`, which is a deliberate decision
  argued in `.just/scripts/fork.sh`
