#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "simplify the mutants daily loop",
  priority: ice(
    impact: 6,
    confidence: 0.9,
    ease: 7,
  ),
  tags: ("tooling", "tests"),
  links: related("adopt-cargo-mutants-mutation-testing.typ")[
    the burn-down this tooling was built for
  ]
    + related("ci-in-diff-mutation-advisory.typ")[
      subsumes the CI question: run it locally, not in CI
    ],
  status: done(
    2026,
    7,
    21,
  )[mutants-diff added and verified end-to-end; ledger, mutants-fix.sh and mutants-fix recipe deleted],
)

== Summary

The burn-down is over, so the scaffolding built for it should go and one daily
driver should remain. Today `just mutants` walks a git-ignored `.mutants-ledger`
that holds 5 entries against 103 candidate files — two of them already stale —
because the actual sweeping was done by `contrib/mutants-fix.sh` and by ad-hoc
`-f` runs that never fed the ledger. The loop's state is therefore fiction: a
fresh clone, or this machine today, reports ~100 files needing a pass and would
re-sweep for hours. Replace the stateful loop with a stateless one scoped to
what you actually changed.

== Scope

- Add `just mutants-diff`: `cargo mutants --in-diff <(git diff <merge-base>)`,
  the pre-PR check. Measured on commit `d5c9540`'s 955-line diff: 31 mutants,
  44s baseline build + 7m07s total at `CARGO_MUTANTS_JOBS=2` on a warm cache,
  31 caught / 0 missed / 0 timeout. Minutes, not hours — and it needs no state
  file, no invalidation rule, and covers new files for free (every line of a new
  file is in the diff).
- Delete the ledger machinery: `.mutants-ledger`, the ~45 lines of bash in the
  `mutants` recipe, and its `.gitignore` entry. Keep `just mutants -f <file>`
  (and passthrough args) as the deliberate full-file sweep.
- Delete `contrib/mutants-fix.sh` and `just mutants-fix`. Its "the model can only
  edit tests" guard is void by construction: line 49 falls back to `sib="$f"`
  when a file has inline tests, handing the model the production file, which is
  how four functions grew return values nobody calls
  (`revert-mutation-driven-api-damage.typ`). It also verifies with `cargo test -p
  mindtape` even when sweeping a `mindtape-cli` file, and commits with a
  hardcoded message and no `Refs:` footer — 63 of this branch's 78 commits
  violate the project's own commit policy because of it.
- Drop the `mutants-needs-human.txt` `.gitignore` entry (the file is gone).
- Rewrite the CONTRIBUTING "Testing" bullets to describe the two remaining
  commands. Keep the two hard-won findings: a `TIMEOUT` is not a verdict in
  either direction, and bound a loop by a `for` over a fixed range rather than
  by the counter a mutant can freeze.

== Decision: no CI job

`cargo mutants --in-diff` in CI was the open question in
`ci-in-diff-mutation-advisory.typ`. Close it as not-worth-it. Every mutant
re-runs a suite that includes subprocess and PTY e2e tests, and the one
operational finding of the whole milestone is that `TIMEOUT` verdicts are
untrustworthy under parallel load — a shared GitHub runner is exactly that
environment. An advisory check that emits unreliable verdicts trains people to
ignore it. The measured 7 minutes above is with a warm local cache on many
cores; a GitHub runner adds the nix install, a cold build and two cores, which
is where an "advisory" turns into the slowest job in the pipeline. The same
signal is available locally, on a machine where a timeout means something, for
the price of one recipe.

== Verification

`just mutants-diff` on a small real diff, `just mutants -f <file>` on one file,
`just check` for the outdatty/docs drift the Justfile edit triggers.
