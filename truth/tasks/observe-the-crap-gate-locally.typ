#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "observe the CRAP gate locally",
  priority: ice(
    impact: 7,
    confidence: 0.9,
    ease: 7,
  ),
  difficulty: 3,
  tags: ("tooling", "gating"),
  estimate: (files: 4),
  links: related(
    "cover-confirm-retire-and-unred-the-crap-gate.typ",
  )[the red gate that went unnoticed because of this]
    + related(
      "ratchet-the-crap-gate-against-a-baseline.typ",
    )[the other half of the same recipe change],
  status: done(
    2026,
    8,
    9,
  )[check-crap skips loudly, check-full is the correct arm],
)

== Summary

`just crap` is CI-enforced and absent from `just check`. Nothing on a developer
machine ever evaluates it, so a green `just check` carries no information about
it at all — which is exactly how it sat red on `main` (`confirm_retire`, CRAP
72) until somebody went looking.

The reason it was left out is real and still holds: the gate needs
`target/coverage/lcov.info`, which only `just cover` produces, and wiring
`just cover` into `just check` puts an instrumented rebuild plus the whole suite
behind every check.

== Scope

Two changes, deliberately both — either alone leaves a hole.

- A conditional `crap` arm inside `just check`. Run the gate when
  `target/coverage/lcov.info` exists and is newer than every source; otherwise
  SKIP LOUDLY, naming `just cover`, and exit 0. This is the mutants arm's
  precedent (`contrib/ask-refactoring/`): absent and clean must never look
  alike. The freshness test is the one `just crap` already performs — extract
  it rather than writing a second one, since two staleness mechanisms that
  disagree are worse than none.
- `just check-full` = `check` + `cover` + `crap`, documented as the pre-PR run.
  This is the arm that is always correct, at its honest price.

Known weakness of the conditional arm, and the reason `check-full` ships with
it: the moment you edit a source the lcov goes stale, so on precisely the
commits that matter the arm skips. It buys the free case (a check after a recent
`just cover`), not the important one. Do not oversell it in the docs — say what
it does and does not catch.

- Measure the price before writing it down. `just cover` ran in 21s warm on the
  maintainer's machine with a hot `target/`; a cold run is a different number
  and CI is a third. Put a measured figure next to `check-full`, not an
  adjective.
- `Justfile` is an `outdatty` `dev-docs` source, so `CONTRIBUTING.md` and
  `AGENTS.md` are forced dependents of this change; reconcile them and run
  `just outdatty-update`.
