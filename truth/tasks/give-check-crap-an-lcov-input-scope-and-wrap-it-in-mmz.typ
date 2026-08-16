#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "give check-crap an lcov input scope and wrap it in mmz",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  difficulty: 2,
  tags: ("tooling", "gating"),
  estimate: (files: 4),
  links: related(
    "split-gates-meta-into-per-script-scopes.typ",
  )[the other mmz-scope change to this manifest],
  status: proposed(2026, 8, 16),
)

== Summary

`check-crap` is the one `just check` arm that runs unwrapped, because a
memoized verdict would outlive the lcov it could not measure. Once mmz can name
a git-ignored path in a scope, that reason is gone: give the rule an `lcov`
scope holding `target/coverage/lcov.info` with `gitignore: false`, wrap the arm
in mmz like every other, and delete the exception from the docs.

== Why the current exception exists

From `docs/contributing/gates.md`: mmz memoizes on file scopes, the lcov is a
build artifact in none of them, so a memoized verdict would survive the lcov
going fresh and that arm would skip forever on unchanged sources. Concretely
`check-crap` records "I could not measure" as a pass; `just cover` then writes
the lcov; sources are unchanged, so mmz reports fresh and the real gate never
runs.

An input scope naming the lcov fixes exactly that: absent and present are
different input states, so the arm goes stale the moment the artifact appears.
Verified against mmz 0.5.1 in a throwaway tree — a scope whose glob currently
matches nothing is legal, the rule runs and skips normally, and the rule goes
stale as soon as the file appears.

== Blocked on

mmz's per-scope `gitignore` key
(`tasks/mmz-per-scope-gitignore-opt-out-for-artifact-paths.typ` in the mmz
repo). `target/` is git-ignored and `.mmz/config.yaml` inherits the
`gitignore: true` default, so the scope is silently empty without it. Flipping
the manifest-level key instead is not an option here: it would walk `target/`
and the 2000 generated files under `examples/large/` this manifest's own
comments say must never churn a rule.

Needs the flake input bumped to the mmz release carrying the key.

== Scope

- `.mmz/config.yaml`: an `lcov` scope with `gitignore: false`, added to
  `check-crap`'s inputs, with the reasoning kept in the comment the way every
  other scope in that file carries its own.
- `Justfile`: `check`'s arm becomes `(mmz "check-crap")` like the rest.
- `.just/scripts/check-crap.sh`: the skip-loudly branch stays. It is still
  right when the lcov has never been written; what changes is that the skip no
  longer sticks once it is.
- `docs/contributing/gates.md` and `docs/src/contributing/gates.typ`: the
  paragraph naming `check-crap` as the deliberate exception goes. Justfile is a
  `dev-docs` outdatty source, so reconcile that group and re-record.
- Prove it end to end rather than by reading the config: record a pass with no
  lcov, write the lcov, and assert the arm goes stale rather than reporting
  fresh.

== Not in scope

`just cover`'s own missing-artifact problem. That is the producer side and it
is mmz's `outputs:` task, not this one.
