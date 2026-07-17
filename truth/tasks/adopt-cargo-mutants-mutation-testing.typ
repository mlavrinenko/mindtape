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
- `cargo-mutants` wired into `flake.nix` devShell (nixpkgs unstable 26.2.0);
  no global install. `just mutants` resolves project-locally.
- Documented as a manual/scoped workflow in CONTRIBUTING.md (Testing) and the
  AGENTS.md dev-tools list; explicitly not a `just check` gate.

== Adoption plan (recommended)

CI story: no workspace-wide nightly sweep (multi-hour, low ROI, and the CLI
files carry `unviable`/timeout noise that makes a green bar hard to hold). Keep
manual scoped `just mutants -f <file>` as the primary discipline. If any CI is
added, make it a per-PR `cargo mutants --in-diff` advisory (non-blocking)
check first — fast, scoped to touched lines — and only promote it to a gate
after the equivalent-mutant floor is stable.

`mutants.toml`: create only when the first CI check lands, not before. It needs
(a) exclude `crates/mindtape-cli/src/main.rs` (mirror the `just crap` exclusion,
same thin-entry-point rationale) and (b) a regex exclusion for each catalogued
equivalent mutant (`numeric.rs:256` and any found later) so a gate has a clean
floor and does not need constant re-triage.

CRAP relationship: not redundant. `just crap` (cheap complexity+undercoverage
proxy) is the prioritisation signal for *where* a mutants pass pays off; mutants
(expensive) proves whether the tests there actually pin behaviour. Pick the next
file to sweep off the CRAP report, not blind.

Rollout scope: burn down the pure-domain top files only, in this order —
`date.rs` (169), `template.rs` (89), `framework.rs` (87), `filter.rs` (79),
`validate.rs` (73). All pure logic, no CLI glue noise, highest signal per
minute. Declare adoption done after these five (plus `numeric.rs`, done). Defer
the CLI files (`add_interactive`, `ls_rules`, cli `recur`) — e2e-heavy, noisier,
lower ROI. Per file: scoped run → triage each survivor into real-gap (regression
test) vs equivalent (catalogue it) vs timeout (bump `--minimum-test-timeout`) →
re-run to confirm → next file.

== Pending

- Confirm the CI decision above before writing any CI config or `mutants.toml`.
- Execute the rollout: `date.rs` first, then the rest of the five. Each pass is
  a separate ~11–25 min scoped run — never foreground a full-workspace sweep.
- Maintain an equivalent-mutant catalogue as files are swept (seed:
  `numeric.rs:256`), for the eventual `mutants.toml` floor.
