#import "@local/mindtape:0.2.0": *

#show: task.with(
  title: "adopt cargo-mutants mutation testing",
  priority: ice(
    impact: 6,
    confidence: 0.8,
    ease: 6,
  ),
  tags: ("tooling", "tests",),
  status: wip(2026, 7, 17)[iterate + ledger loop landed; burning down files],
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

A dev-aid loop, not a gate — kept out of `just check`. `just mutants` runs one
bounded step: the next suitable file whose source or `_tests.rs` sibling changed
since it was last swept clean, then reports how many remain. Repeat across
sessions until every file is clean.

- Engine: `cargo mutants --iterate`, which skips mutants caught in prior runs so
  re-runs stay cheap. It is a heuristic (matches file/line/col/desc), so a final
  result is only trusted after a run without `--iterate`.
- File set: auto-discovered via `cargo mutants --list-files` — no hand-list to
  drift from `./src`. "Suitable" is a predicate, set by `.cargo/mutants.toml`
  excludes, not an enumeration.
- Next-file + remaining count: a git-ignored `.mutants-ledger` maps each
  swept-clean file to the git-blob hash of it plus its `_tests.rs` sibling
  (tight per-file invalidation — editing either re-stales just that file). No
  mmz: cargo-mutants owns invalidation.
- Not every survivor is a bug: `numeric.rs:256` (`<` vs `<=` after an
  exact-match short-circuit) is equivalent — 100% is not the target.

Why not mmz: mmz gives file-level content-hash skip, but a per-file rule per
target hand-listed in `.mmz/config.yaml` drifts from the tree. `--iterate` plus
`--list-files` covers the incremental need natively with zero per-file wiring.
The general mmz gap (glob-fanned per-file rules) is filed separately as
`mmz-per-file-command-cache`.

== Done in this pass

- `.cargo/mutants.toml`: excludes `**/main.rs` (mirrors the `just crap`
  exclusion) and the `numeric.rs` `<`→`<=` equivalent mutant via `exclude_re`;
  runs tests with `--features rrule` (matches `just check`); `minimum_test_timeout
  = 60`. Tracked despite `.gitignore` ignoring the rest of `.cargo/`.
- `just mutants` rewritten into the `--iterate` + ledger loop above; `-f <file>`
  and arbitrary passthrough retained.
- `.mutants-ledger` and the existing `mutants.out` git-ignored.
- `cargo-mutants` wired into `flake.nix` devShell (nixpkgs unstable 26.2.0);
  `just mutants` resolves project-locally, no global install.
- Documented in CONTRIBUTING.md (Testing); not a `just check` gate.
- Earlier: closed the 4 real gaps the trial found in `numeric.rs` (band `min >
  max`, band `min == max` boundary, anchor out-of-range, point-below-lowest
  derivation); only survivor is the excluded equivalent at `numeric.rs:256`.

== CI story

No workspace-wide nightly sweep (multi-hour, low ROI, CLI files carry
`unviable`/timeout noise). The loop is the primary discipline. If CI is ever
added, a per-PR `cargo mutants --in-diff` advisory (non-blocking) check is the
candidate — fast, scoped to touched lines — promoted to a gate only once the
equivalent-mutant floor in `.cargo/mutants.toml` is stable. CRAP stays
complementary: `just crap` is the cheap proxy for *where* a pass pays off;
mutants proves whether tests there actually pin behaviour.

== Pending

- Burn down the pure-domain files across sessions via `just mutants`, roughly
  highest-signal first (`date.rs`, `template.rs`, `framework.rs`, `filter.rs`,
  `validate.rs`, `sort.rs`, `model.rs`, ...). Never foreground a full sweep.
- Grow the `exclude_re` equivalent-mutant catalogue in `.cargo/mutants.toml` as
  files are swept (seed: the `numeric.rs` `<`→`<=` mutant).
- Reassess whether a CI `--in-diff` advisory earns its keep after a few files.
