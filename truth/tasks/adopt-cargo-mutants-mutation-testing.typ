#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "adopt cargo-mutants mutation testing",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("tooling", "tests",),
  status: proposed(2026, 7, 17),
)

== Summary

Add mutation testing (cargo-mutants) as an on-demand quality signal for the test
suite. Line coverage and the CRAP gate prove a line ran; a mutant proves a test
would fail if the line's behaviour broke. Surviving (`MISSED`) mutants are the
assertion gaps that a refactor could silently walk through. Use it to vet the
modules touched by a refactor before starting.

== Why

967 tests, all green, well structured (sibling `_tests.rs`, real `assert_cmd`
e2e, CQRS read-no-write invariant enforced). Strong, but count and coverage do
not measure whether assertions pin behaviour. A scoped trial on `numeric.rs`
(85 mutants) scored 66 caught / 6 missed / 9 unviable / 4 timeout; the misses
were real edge-branch gaps, not noise.

== Operational model

- Not an inner-loop gate: ~11 min for 85 mutants (2400+ workspace-wide, hours).
  Keep it out of `just check`.
- `just mutants -f <file>`: scope to the module under refactor; surviving
  mutants there are the tests that will not guard the change.
- Candidate later: a nightly / on-label CI job over the whole workspace, or an
  `--in-diff` run against the PR diff.
- Not every survivor is a bug: `numeric.rs:256` (`< ` vs `<=` after an
  exact-match short-circuit) is an equivalent mutant — 100% is not the target.

== Done in this pass

- `just mutants` recipe (`--minimum-test-timeout 60` to avoid false timeouts).
- `mutants.out` git-ignored.
- Closed the 4 real gaps the trial found in `numeric.rs` with regression tests:
  band `min > max`, band `min == max` (the `>` vs `>=` boundary), anchor
  out-of-range, point-below-lowest-anchor derivation. Re-run confirms the only
  survivor is the equivalent mutant at `numeric.rs:256`.

== Pending

- Wire `cargo-mutants` into `flake.nix` devShell. Deferred: cache.nixos.org was
  unreachable at trial time, so it was installed via `cargo install --locked`
  against the project's pinned toolchain instead; the cache reachability issue
  is being addressed separately at the dotfiles level. Once wired, note the
  tool in CONTRIBUTING.md if a mutation-testing workflow is documented there.
- Decide the CI story (nightly full sweep vs per-PR `--in-diff`). If gated, add
  a `mutants.toml` excluding the `numeric.rs:256` equivalent mutant so the gate
  has a clean floor.
