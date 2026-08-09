#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "ratchet the CRAP gate against a baseline",
  priority: ice(
    impact: 7,
    confidence: 0.8,
    ease: 5,
  ),
  difficulty: 5,
  tags: ("tooling", "gating"),
  estimate: (files: 6),
  links: related(
    "sweep-crap-scores-into-tasks.typ",
  )[the cancelled band this replaces, and the measurement that rules it out]
    + related(
      "observe-the-crap-gate-locally.typ",
    )[the other half of the same recipe change],
  status: done(
    2026,
    8,
    9,
  )[baseline minted green; regression detection verified by tampering],
)

== Summary

The CRAP gate answers exactly one question — is any function above 30 — and it
is blind in two directions. Below 30 it is silent: `hval_kind` is CC 8 at 40%
coverage, scores 21.8, passes. And it has no memory: drop that same function to
20% coverage and it scores 26, still passing, with nothing anywhere recording
that it got worse.

The obvious fix is an advisory band below 30, and that is the one thing
measurement rules out — `sweep-crap-scores-into-tasks` cancelled it because
above ~90% coverage CRAP degenerates into cyclomatic complexity, so a band
reports well-covered complex functions and misses undertested ones.

`cargo crap --baseline <FILE> --fail-regression` fixes the second blindness
instead, and it is immune to the degeneracy that killed the band: it never reads
the absolute score, only the delta against a committed baseline. A rise means
cyclomatic complexity went up or coverage went down. Both are real regressions
at any absolute level, which is precisely what the band could not say.

== Scope

- Mint `crap-baseline.json` with `--format json --sort file --output` and commit
  it. `--sort file` is documented as the stable ordering for a committed
  baseline; `--sort crap` reshuffles the whole file whenever any score moves,
  which turns every diff into noise.
- Mint it only against a green gate. A baseline records today as acceptable, so
  minting while something is red freezes the red. `confirm_retire` was that
  case and is now fixed; assert `just crap` exits 0 before writing the file.
- Keep `--fail-above 30` alongside `--fail-regression`. The ratchet stops new
  debt and the threshold still catches a single catastrophic function; a ratchet
  alone would accept anything as long as it arrived all at once.
- Pick `--epsilon` from observation, not from the default. CRAP is a float, so
  ordinary coverage wobble moves a score by hundredths and the default (0.01)
  may flag it as a regression. Re-run `just cover` twice without touching a
  source and measure the spread before choosing.
- Decide and document how a legitimate increase lands. Adding a branch to a
  function is normal; the answer is a baseline refresh in the same commit, so
  the recipe needs a `crap-baseline` (or equivalent) update verb and
  CONTRIBUTING has to say when to run it. A ratchet with no documented release
  valve gets bypassed instead of refreshed.
- Consider tracking the baseline as an `outdatty` dependent of the sources it
  scores, so a stale baseline is caught the way other drift is. Decide it, do
  not default into it — the group would fire on every source edit, which may be
  noise rather than signal.
- `Justfile` is an `outdatty` `dev-docs` source: `CONTRIBUTING.md` and
  `AGENTS.md` are forced dependents, plus `just outdatty-update`. The CI
  `coverage` job runs `just crap` and inherits this for free.
