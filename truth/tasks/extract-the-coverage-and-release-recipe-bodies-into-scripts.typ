#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "extract the coverage and release recipe bodies into scripts",
  priority: ice(
    impact: 7,
    confidence: 0.75,
    ease: 4,
  ),
  difficulty: 5,
  tags: ("tooling",),
  estimate: (files: 15),
  links: depends-on(
    "move-the-just-module-and-scripts-into-a-dot-just-folder.typ",
  ),
  status: proposed(2026, 8, 14),
)

== Summary

Same extraction as the gate-bodies task, for the coverage, mutation, install and
release recipes.

== In scope

- `cover`
- `crap`
- `crap-baseline`
- `_lcov-stale-reason`
- `mutants`
- `mutants-diff`
- `install-lib`
- `release`

== Out of scope

`uninstall-lib` and `install-hooks` are four lines each and stay inline.

== Two things not to lose

`_lcov-stale-reason` is deliberately a recipe rather than a copied block: one
staleness implementation, two callers with opposite policies (`crap` treats a
stale lcov as a hard error, `check-crap` as a reason to skip). It also always
exits 0 on purpose, so that a deliberate skip does not print just's own
`error: Recipe ... failed` line and read on screen like a broken gate. Both
properties survive extraction only if the script keeps them — the exit-0
discipline in particular is easy to "fix" into a bug.

`CRAP_EXCLUDES` is a Justfile variable interpolated into `crap`,
`crap-baseline`, and (via `_crap-excludes`) `contrib/ask-refactoring/undertested.sh`.
It stays a Justfile variable and is passed into the extracted scripts as an
argument or an environment variable. Do not copy the list into a script — the
whole reason it exists as one variable is that a baseline minted over a
different file set than the gate scores reports every difference as a
regression.

== Scope

- One script per recipe, executable, carrying the `set -eo pipefail` contract
  each body already declares. Never `set -u`.
- `release` is `[confirm(...)]`-guarded and pushes tags. Its guard stays on the
  recipe; the script must not be independently runnable in a way that skips it.
  Verify the confirmation still fires after extraction.
- Comment relocation is a separate task. Leave the comments in place here.
- `outdatty.yaml`: add the new scripts as `dev-docs` sources and re-confirm the
  seven dependents with `just outdatty-update`.
- `docs/src/contributing/coverage-crap.typ` and
  `docs/src/contributing/mutation-testing.typ` describe these recipes by hand.
  They are `dev-docs` dependents, so they must be reviewed — check whether any
  of their prose names a body line that has moved.
